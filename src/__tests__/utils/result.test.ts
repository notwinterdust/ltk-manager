import type { Result } from "@/utils/result";
import {
  andThen,
  isErr,
  isOk,
  map,
  mapErr,
  match,
  unwrap,
  unwrapOr,
  unwrapOrElse,
} from "@/utils/result";

const ok = <T>(value: T): Result<T, string> => ({ ok: true, value });
const err = <T>(error: string): Result<T, string> => ({ ok: false, error });

describe("isOk", () => {
  it("returns true for Ok", () => {
    expect(isOk(ok(42))).toBe(true);
  });

  it("returns false for Err", () => {
    expect(isOk(err("fail"))).toBe(false);
  });
});

describe("isErr", () => {
  it("returns true for Err", () => {
    expect(isErr(err("fail"))).toBe(true);
  });

  it("returns false for Ok", () => {
    expect(isErr(ok(42))).toBe(false);
  });
});

describe("unwrap", () => {
  it("returns the value for Ok", () => {
    expect(unwrap(ok(42))).toBe(42);
  });

  it("throws the error for Err", () => {
    expect(() => unwrap(err("fail"))).toThrow("fail");
  });
});

describe("unwrapOr", () => {
  it("returns the value for Ok", () => {
    expect(unwrapOr(ok(42), 0)).toBe(42);
  });

  it("returns the default for Err", () => {
    expect(unwrapOr(err("fail"), 0)).toBe(0);
  });
});

describe("unwrapOrElse", () => {
  it("returns the value for Ok", () => {
    expect(unwrapOrElse(ok(42), () => 0)).toBe(42);
  });

  it("calls the function for Err", () => {
    expect(unwrapOrElse(err("fail"), (e) => e.length)).toBe(4);
  });
});

describe("map", () => {
  it("transforms the Ok value", () => {
    const result = map(ok(2), (v) => v * 3);
    expect(result).toEqual({ ok: true, value: 6 });
  });

  it("passes through Err unchanged", () => {
    const result = map(err<number>("fail"), (v) => v * 3);
    expect(result).toEqual({ ok: false, error: "fail" });
  });
});

describe("mapErr", () => {
  it("transforms the Err value", () => {
    const result = mapErr(err("fail"), (e) => e.toUpperCase());
    expect(result).toEqual({ ok: false, error: "FAIL" });
  });

  it("passes through Ok unchanged", () => {
    const result = mapErr(ok(42), (e: string) => e.toUpperCase());
    expect(result).toEqual({ ok: true, value: 42 });
  });
});

describe("andThen", () => {
  it("chains Ok results", () => {
    const result = andThen(ok(2), (v) => ok(v * 3));
    expect(result).toEqual({ ok: true, value: 6 });
  });

  it("short-circuits on Err", () => {
    const result = andThen(err<number>("fail"), (v) => ok(v * 3));
    expect(result).toEqual({ ok: false, error: "fail" });
  });

  it("propagates Err from the chain function", () => {
    const result = andThen(ok(2), () => err<number>("chained error"));
    expect(result).toEqual({ ok: false, error: "chained error" });
  });
});

describe("match", () => {
  it("calls ok handler for Ok", () => {
    const result = match(ok(42), {
      ok: (v) => `value: ${v}`,
      err: (e) => `error: ${e}`,
    });
    expect(result).toBe("value: 42");
  });

  it("calls err handler for Err", () => {
    const result = match(err("fail"), {
      ok: (v) => `value: ${v}`,
      err: (e) => `error: ${e}`,
    });
    expect(result).toBe("error: fail");
  });
});
