import { expect, test } from "vitest";
import { apiPath } from "../store.ts";
import {
    userLoginUrl, userRegisterUrl, imageUploadUrl,
    imageGalleryUrl, imageUrl,
} from "./api.ts";

test("apiUrls", () => {
    let url: string;

    // Test user login URL
    url = userLoginUrl();
    expect(url).toEqual(apiPath + "/login");

    // Test user register URL
    url = userRegisterUrl();
    expect(url).toEqual(apiPath + "/register");

    // Test image upload URL
    url = imageUploadUrl();
    expect(url).toEqual(apiPath + "/images");

    // Test image gallery URL
    url = imageGalleryUrl(1, 12);
    expect(url).toEqual(apiPath + "/images?page=1&limit=12");

    // Test image URL
    url = imageUrl("4d50ac44-a78c-4bc3-9e6f-1c2e7a50e897");
    expect(url).toEqual(apiPath + "/images/4d50ac44-a78c-4bc3-9e6f-1c2e7a50e897");

    // Test image URL with unsafe characters
    url = imageUrl("4d50ac44-a78c-4bc3-9e6f-1c2e7a50e897?annihilate=true");
    expect(url).toEqual(apiPath + "/images/4d50ac44-a78c-4bc3-9e6f-1c2e7a50e897%3Fannihilate%3Dtrue");
});

