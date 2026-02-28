import { mutationFn, queryFn, queryFnWithArgs, unwrapForQuery } from "@/utils/query";
import type { Result } from "@/utils/result";

const ok = <T>(value: T): Result<T, string> => ({ ok: true, value });
const err = <T>(error: string): Result<T, string> => ({ ok: false, error });

describe("unwrapForQuery", () => {
  it("returns the value for Ok", () => {
    expect(unwrapForQuery(ok(42))).toBe(42);
  });

  it("throws the error for Err", () => {
    expect(() => unwrapForQuery(err("fail"))).toThrow("fail");
  });
});

describe("queryFn", () => {
  it("returns a function that unwraps Ok results", async () => {
    const apiFn = vi.fn(() => Promise.resolve(ok("data")));
    const wrapped = queryFn(apiFn);
    await expect(wrapped()).resolves.toBe("data");
    expect(apiFn).toHaveBeenCalledOnce();
  });

  it("returns a function that throws on Err results", async () => {
    const apiFn = vi.fn(() => Promise.resolve(err<string>("fail")));
    const wrapped = queryFn(apiFn);
    await expect(wrapped()).rejects.toBe("fail");
  });
});

describe("queryFnWithArgs", () => {
  it("passes arguments to the wrapped function", async () => {
    const apiFn = vi.fn((id: string) => Promise.resolve(ok(`data-${id}`)));
    const wrapped = queryFnWithArgs(apiFn, "abc");
    await expect(wrapped()).resolves.toBe("data-abc");
    expect(apiFn).toHaveBeenCalledWith("abc");
  });

  it("passes multiple arguments", async () => {
    const apiFn = vi.fn((a: string, b: number) => Promise.resolve(ok(`${a}-${b}`)));
    const wrapped = queryFnWithArgs(apiFn, "x", 42);
    await expect(wrapped()).resolves.toBe("x-42");
    expect(apiFn).toHaveBeenCalledWith("x", 42);
  });

  it("throws on Err results", async () => {
    const apiFn = vi.fn((_id: string) => Promise.resolve(err<string>("not found")));
    const wrapped = queryFnWithArgs(apiFn, "abc");
    await expect(wrapped()).rejects.toBe("not found");
  });
});

describe("mutationFn", () => {
  it("returns a function that unwraps Ok results", async () => {
    const apiFn = vi.fn((vars: { name: string }) => Promise.resolve(ok(`created-${vars.name}`)));
    const wrapped = mutationFn(apiFn);
    await expect(wrapped({ name: "test" })).resolves.toBe("created-test");
    expect(apiFn).toHaveBeenCalledWith({ name: "test" });
  });

  it("throws on Err results", async () => {
    const apiFn = vi.fn((_vars: { name: string }) => Promise.resolve(err<string>("fail")));
    const wrapped = mutationFn(apiFn);
    await expect(wrapped({ name: "test" })).rejects.toBe("fail");
  });
});
