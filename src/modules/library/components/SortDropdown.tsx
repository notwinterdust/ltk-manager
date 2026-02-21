import { SelectField } from "@/components";
import { type SortConfig, useLibraryFilterStore } from "@/stores";

const SORT_OPTIONS = [
  { value: "manual:asc", label: "Manual Order" },
  { value: "name:asc", label: "Name (A-Z)" },
  { value: "name:desc", label: "Name (Z-A)" },
  { value: "installedAt:desc", label: "Newest First" },
  { value: "installedAt:asc", label: "Oldest First" },
  { value: "enabled:asc", label: "Enabled First" },
];

function toValue(sort: SortConfig): string {
  return `${sort.field}:${sort.direction}`;
}

function fromValue(value: string): SortConfig {
  const [field, direction] = value.split(":") as [SortConfig["field"], SortConfig["direction"]];
  return { field, direction };
}

export function SortDropdown() {
  const { sort, setSort } = useLibraryFilterStore();

  return (
    <SelectField
      options={SORT_OPTIONS}
      value={toValue(sort)}
      onValueChange={(v) => v && setSort(fromValue(v))}
      triggerClassName="w-40 py-1.5 text-xs"
    />
  );
}
