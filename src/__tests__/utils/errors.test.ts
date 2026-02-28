import type { AppError } from "@/lib/bindings";
import { getInvalidPathContext, getModNotFoundContext, hasErrorCode } from "@/utils/errors";

function makeError(code: string, context?: unknown): AppError {
  return {
    code: code as AppError["code"],
    message: `Test error: ${code}`,
    context,
  };
}

describe("hasErrorCode", () => {
  it("returns true when codes match", () => {
    const error = makeError("INVALID_PATH");
    expect(hasErrorCode(error, "INVALID_PATH")).toBe(true);
  });

  it("returns false when codes differ", () => {
    const error = makeError("IO");
    expect(hasErrorCode(error, "INVALID_PATH")).toBe(false);
  });
});

describe("getInvalidPathContext", () => {
  it("returns context for INVALID_PATH with valid context", () => {
    const error = makeError("INVALID_PATH", { path: "/some/path" });
    const ctx = getInvalidPathContext(error);
    expect(ctx).toEqual({ path: "/some/path" });
  });

  it("returns undefined for wrong error code", () => {
    const error = makeError("IO", { path: "/some/path" });
    expect(getInvalidPathContext(error)).toBeUndefined();
  });

  it("returns undefined when context is missing", () => {
    const error = makeError("INVALID_PATH");
    expect(getInvalidPathContext(error)).toBeUndefined();
  });

  it("returns undefined for malformed context", () => {
    const error = makeError("INVALID_PATH", { wrongKey: 123 });
    expect(getInvalidPathContext(error)).toBeUndefined();
  });
});

describe("getModNotFoundContext", () => {
  it("returns context for MOD_NOT_FOUND with valid context", () => {
    const error = makeError("MOD_NOT_FOUND", { modId: "test-mod" });
    const ctx = getModNotFoundContext(error);
    expect(ctx).toEqual({ modId: "test-mod" });
  });

  it("returns undefined for wrong error code", () => {
    const error = makeError("IO", { modId: "test-mod" });
    expect(getModNotFoundContext(error)).toBeUndefined();
  });

  it("returns undefined when context is missing", () => {
    const error = makeError("MOD_NOT_FOUND");
    expect(getModNotFoundContext(error)).toBeUndefined();
  });

  it("returns undefined for malformed context", () => {
    const error = makeError("MOD_NOT_FOUND", { wrongKey: 123 });
    expect(getModNotFoundContext(error)).toBeUndefined();
  });
});
