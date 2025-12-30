use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use models::{ContentType, Image, ImageInfo, ImageVersion};

/// Insert image record into the database.
pub async fn insert_image(
    db: &PgPool,
    id: &Uuid,
    name: &str,
    content_type: ContentType,
    username: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO image (id, name, content_type, username)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (name, username) DO NOTHING
        "#,
        id,
        name,
        content_type as i32,
        username,
    )
    .execute(db)
    .await?;

    Ok(())
}

/// Insert image version data into the database.
pub async fn insert_image_version(
    db: &PgPool,
    image_id: &Uuid,
    version: &str,
    dimensions: (u32, u32),
    size: usize,
) -> Result<()> {
    let version_option: Option<String> = sqlx::query_scalar!(
        r#"
        INSERT INTO image_version (
            image_id, version, current, width, height, size
        )
        VALUES ($1, $2, TRUE, $3, $4, $5)
        RETURNING version
        "#,
        image_id,
        version,
        dimensions.0 as i32,
        dimensions.1 as i32,
        size as i64,
    )
    .fetch_optional(db)
    .await?;

    // Unset the `current` flag for any old versions
    unset_current_version_flags(db, image_id, &version_option).await?;

    Ok(())
}

/// Retrieve database data for a single image.
pub async fn find_image(
    db: &PgPool,
    id: &Uuid,
    username: &str,
) -> Result<Option<ImageInfo>> {
    let image = sqlx::query_as::<_, ImageInfo>(
        r#"
        SELECT i.id, i.name, i.username, i.content_type, v.version
        FROM image AS i
        LEFT JOIN image_version AS v
            ON v.image_id = i.id
        WHERE v.current
            AND i.id = $1
            AND i.username = $2
        "#,
    )
    .bind(id)
    .bind(username)
    .fetch_optional(db)
    .await?;

    Ok(image)
}

/// Retrieve database data (including extra version data)
/// for a single image.
pub async fn find_image_with_version_info(
    db: &PgPool,
    id: &Uuid,
    username: &str,
) -> Result<Option<Image>> {
    let image = sqlx::query_as::<_, Image>(
        r#"
        WITH image_info AS (
            SELECT * FROM image
            WHERE id = $1 AND username = $2
        ),
        versions AS (
            SELECT ROW_NUMBER() OVER (
                ORDER BY ts
            ) AS idx, *
            FROM image_version
            WHERE image_id = (SELECT id FROM image_info)
            ORDER BY ts DESC
        ),
        current_version AS (
            SELECT * FROM versions
            WHERE current
                AND image_id = (SELECT id FROM image_info)
        ),
        version_count AS (
            SELECT COUNT(1) AS version_count
            FROM versions
        )
        SELECT i.id, i.name, i.content_type, i.created_at,
            v.ts AS last_modified, v.version,
            v.width, v.height, v.size, vc.version_count,
            v.idx AS version_index,
            v.idx = vc.version_count AS latest_version,
            v.idx = 1 AS initial_version
        FROM image_info AS i
        LEFT JOIN current_version AS v
            ON TRUE
        LEFT JOIN version_count AS vc
            ON TRUE
        "#,
    )
    .bind(id)
    .bind(username)
    .fetch_optional(db)
    .await?;

    Ok(image)
}

/// Retrieve database data for a single image.
pub async fn find_image_id_by_name(
    db: &PgPool,
    name: &str,
    username: &str,
) -> Result<Option<Uuid>> {
    let image_id: Option<Uuid> = sqlx::query_scalar!(
        "SELECT id FROM image WHERE name = $1 and username = $2",
        name,
        username,
    )
    .fetch_optional(db)
    .await?;

    Ok(image_id)
}

/// Retrieve database data for all of the given user's images.
pub async fn find_all_images(
    db: &PgPool,
    username: &str,
) -> Result<Vec<Image>> {
    let images = sqlx::query_as::<_, Image>(
        r#"
        WITH images AS (
            SELECT * FROM image WHERE username = $1
        ),
        versions AS (
            SELECT ROW_NUMBER() OVER (
                PARTITION BY image_id ORDER BY ts
            ) AS idx, *
            FROM image_version
            WHERE image_id IN (SELECT id FROM images)
            ORDER BY ts DESC
        ),
        current_version AS (
            SELECT * FROM versions
            WHERE current
                AND image_id IN (SELECT id FROM images)
        ),
        version_count AS (
            SELECT image_id, COUNT(1) AS version_count
            FROM versions
            GROUP BY image_id
        )
        SELECT i.id, i.name, i.content_type, i.created_at,
            v.ts AS last_modified, v.version,
            v.width, v.height, v.size, vc.version_count,
            v.idx AS version_index,
            v.idx = vc.version_count AS latest_version,
            v.idx = 1 AS initial_version
        FROM images AS i
        LEFT JOIN current_version AS v
            ON v.image_id = i.id
        LEFT JOIN version_count AS vc
            ON vc.image_id = i.id
        "#,
    )
    .bind(username)
    .fetch_all(db)
    .await?;

    Ok(images)
}

/// Delete an image.
pub async fn delete_image(
    db: &PgPool,
    image_id: &Uuid,
) -> Result<()> {
    sqlx::query!(
        "DELETE FROM image WHERE id = $1",
        image_id,
    )
    .execute(db)
    .await?;

    Ok(())
}

/// Revert an image to its previous version.
pub async fn revert_image_version(
    db: &PgPool,
    image_id: &Uuid,
) -> Result<Option<String>> {
    if let Some(prior_version) = get_current_version(db, image_id).await? {
        // Set the `current` flag for the next most recent
        // version and return the version id
        if let Ok(version) = sqlx::query_scalar!(
            r#"
            UPDATE image_version SET current = TRUE
            WHERE version = (
                SELECT version FROM image_version
                WHERE ts < $1
                    AND image_id = $2
                    AND version <> $3
                ORDER BY ts DESC LIMIT 1
            )
            RETURNING version
            "#,
            prior_version.ts,
            image_id,
            prior_version.version,
        )
        .fetch_optional(db)
        .await {
            unset_current_version_flags(db, image_id, &version).await?;
            return Ok(version);
        }
    }

    Ok(None)
}

/// Restore an image to its more recent version.
pub async fn restore_image_version(
    db: &PgPool,
    image_id: &Uuid,
) -> Result<Option<String>> {
    if let Some(prior_version) = get_current_version(db, image_id).await? {
        // Set the `current` flag for the more recent
        // version and return the version id
        if let Ok(version) = sqlx::query_scalar!(
            r#"
            UPDATE image_version SET current = TRUE
            WHERE version = (
                SELECT version FROM image_version
                WHERE ts > $1
                    AND image_id = $2
                    AND version <> $3
                ORDER BY ts ASC LIMIT 1
            )
            RETURNING version
            "#,
            prior_version.ts,
            image_id,
            prior_version.version,
        )
        .fetch_optional(db)
        .await {
            unset_current_version_flags(db, image_id, &version).await?;
            return Ok(version);
        }
    }

    Ok(None)
}

/// Update the name of an image.
pub async fn rename_image(
    db: &PgPool,
    image_id: &Uuid,
    new_name: &str,
) -> Result<Option<String>> {
    let image_name: Option<String> = sqlx::query_scalar!(
        r#"
        UPDATE image SET name = $1
        WHERE id = $2
        RETURNING name
        "#,
        new_name,
        image_id,
    )
    .fetch_one(db)
    .await?;

    Ok(image_name)
}

/// Unset the `current` flag for any old versions of an image.
async fn unset_current_version_flags(
    db: &PgPool,
    image_id: &Uuid,
    version: &Option<String>,
) -> Result<()> {
    if let Some(version_id) = version {
        sqlx::query!(
            r#"
            UPDATE image_version SET current = FALSE
            WHERE image_id = $1 AND version <> $2
            "#,
            image_id,
            &version_id,
        )
        .execute(db)
        .await?;
    }

    Ok(())
}

/// Get the current version info for an image.
async fn get_current_version(
    db: &PgPool,
    image_id: &Uuid,
) -> Result<Option<ImageVersion>> {
    let version_info = sqlx::query_as::<_, ImageVersion>(
        r#"
        SELECT version, ts FROM image_version
        WHERE current AND image_id = $1
        "#,
    )
    .bind(image_id)
    .fetch_optional(db)
    .await?;

    Ok(version_info)
}
