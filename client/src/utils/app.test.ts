import { expect, test } from "vitest";
import {
    getFileExtension, getFileStem, truncateFileName,
    formatImageType, formatFileSize, formatDate,
} from "./app.ts";

test("getFileExtension", () => {
    let extension: string;

    // Test PNG filename
    extension = getFileExtension("something.png");
    expect(extension).toEqual("png");

    // Test filename without extension
    extension = getFileExtension("something");
    expect(extension).toEqual("jpg");
});

test("getFileStem", () => {
    let stem: string;

    // Test PNG filename
    stem = getFileStem("something.png");
    expect(stem).toEqual("something");

    // Test filename without extension
    stem = getFileStem("something");
    expect(stem).toEqual("something");
});

test("truncateFileName", () => {
    let filename: string;

    // Test filename having 26 or more characters
    filename = truncateFileName("abcdefghijklmnopqrstuvwxyz.png");
    expect(filename).toEqual("abcdefghijklmnopqrstuv... .png");

    // Test filename having less than 26 characters
    filename = truncateFileName("something.png");
    expect(filename).toEqual("something.png");
});

test("formatImageType", () => {
    let imageType: string;

    // Test image content type
    imageType = formatImageType("image/png");
    expect(imageType).toEqual("PNG");

    // Test non-image content type
    imageType = formatImageType("audio/mpeg");
    expect(imageType).toEqual("UNKNOWN");
});

test("formatFileSize", () => {
    let fileSize: string;

    // Test file size in bytes
    fileSize = formatFileSize(1000);
    expect(fileSize).toEqual("1000 B");

    // Test file size in kibibytes
    fileSize = formatFileSize(1024);
    expect(fileSize).toEqual("1.0 KiB");

    // Test file size in mebibytes
    fileSize = formatFileSize(1024 * 1024);
    expect(fileSize).toEqual("1.0 MiB");
});

test("formatDate", () => {
    let dateString: string;

    // Test AM datetime
    dateString = formatDate("2026-06-15 09:23:42.983712-04");
    expect(dateString).toEqual("6/15/2026, 9:23:42 AM");

    // Test PM datetime
    dateString = formatDate("2026-06-15 15:50:56.274837-04");
    expect(dateString).toEqual("6/15/2026, 3:50:56 PM");
});

