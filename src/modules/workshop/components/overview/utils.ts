import type { WorkshopAuthor } from "@/lib/tauri";

/**
 * Parse a comma-separated champions string into an array of trimmed, non-empty names.
 */
export function parseChampionsText(text: string): string[] {
  return text
    .split(",")
    .map((c) => c.trim())
    .filter(Boolean);
}

/**
 * Filter out authors with empty names.
 */
export function filterEmptyAuthors(authors: WorkshopAuthor[]): WorkshopAuthor[] {
  return authors.filter((a) => a.name.trim());
}

/**
 * Update a single field on an author at a given index, returning a new array.
 */
export function updateAuthorAt(
  authors: WorkshopAuthor[],
  index: number,
  field: "name" | "role",
  value: string,
): WorkshopAuthor[] {
  const updated = [...authors];
  updated[index] = { ...updated[index], [field]: value };
  return updated;
}

/**
 * Remove the author at a given index, returning a new array.
 */
export function removeAuthorAt(authors: WorkshopAuthor[], index: number): WorkshopAuthor[] {
  return authors.filter((_, i) => i !== index);
}

/**
 * Append a blank author entry.
 */
export function appendAuthor(authors: WorkshopAuthor[]): WorkshopAuthor[] {
  return [...authors, { name: "", role: "" }];
}
