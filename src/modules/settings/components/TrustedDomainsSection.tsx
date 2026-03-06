import { useState } from "react";
import { LuPlus, LuX } from "react-icons/lu";

import { Button, SectionCard } from "@/components";
import type { Settings } from "@/lib/tauri";

interface TrustedDomainsSectionProps {
  settings: Settings;
  onSave: (settings: Settings) => void;
}

export function TrustedDomainsSection({ settings, onSave }: TrustedDomainsSectionProps) {
  const [newDomain, setNewDomain] = useState("");

  const domains = settings.trustedDomains ?? [];

  function addDomain() {
    const trimmed = newDomain.trim().toLowerCase();
    if (!trimmed || domains.includes(trimmed)) return;
    onSave({ ...settings, trustedDomains: [...domains, trimmed] });
    setNewDomain("");
  }

  function removeDomain(domain: string) {
    onSave({ ...settings, trustedDomains: domains.filter((d) => d !== domain) });
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      addDomain();
    }
  }

  return (
    <SectionCard title="Trusted Mod Providers">
      <div className="space-y-3">
        <p className="text-sm text-surface-400">
          Only mods from these domains can be installed via one-click links. Remove all domains to
          allow any source.
        </p>

        <div className="space-y-1.5">
          {domains.map((domain) => (
            <div
              key={domain}
              className="flex items-center justify-between rounded-md bg-surface-800 px-3 py-2"
            >
              <span className="text-sm text-surface-200">{domain}</span>
              <button
                type="button"
                onClick={() => removeDomain(domain)}
                className="rounded p-1 text-surface-400 hover:bg-surface-700 hover:text-surface-200"
              >
                <LuX className="h-3.5 w-3.5" />
              </button>
            </div>
          ))}
        </div>

        <div className="flex gap-2">
          <input
            type="text"
            value={newDomain}
            onChange={(e) => setNewDomain(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="e.g. example.com"
            className="flex-1 rounded-md border border-surface-600 bg-surface-800 px-3 py-1.5 text-sm text-surface-200 placeholder:text-surface-500 focus:border-brand-500 focus:outline-none"
          />
          <Button variant="ghost" size="sm" onClick={addDomain} disabled={!newDomain.trim()}>
            <LuPlus className="h-4 w-4" />
            Add
          </Button>
        </div>
      </div>
    </SectionCard>
  );
}
