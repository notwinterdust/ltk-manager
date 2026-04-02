export interface SkinhackFlag {
  reason: string;
}

const BLOCKED_AUTHORS = ["rose", "darkseal.org"];

const BLOCKED_DESCRIPTION_PATTERNS = ["https://github.com/alban1911/rose"];

export function checkModForSkinhack(mod: {
  authors: string[];
  description?: string | null;
}): SkinhackFlag | null {
  const authorsLower = mod.authors.map((a) => a.toLowerCase());

  for (const blocked of BLOCKED_AUTHORS) {
    if (authorsLower.some((author) => author === blocked)) {
      return { reason: `Known skinhack source: "${blocked}"` };
    }
  }

  if (mod.description) {
    const descLower = mod.description.toLowerCase();
    for (const pattern of BLOCKED_DESCRIPTION_PATTERNS) {
      if (descLower.includes(pattern)) {
        return { reason: `Known skinhack reference: "${pattern}"` };
      }
    }
  }

  return null;
}
