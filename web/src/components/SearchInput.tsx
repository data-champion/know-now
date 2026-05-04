interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}

export function SearchInput({ value, onChange, placeholder }: SearchInputProps) {
  return (
    <input
      type="search"
      role="searchbox"
      aria-label={placeholder ?? "Search"}
      className="kn-search"
      value={value}
      onChange={(e) => { onChange(e.target.value); }}
      placeholder={placeholder ?? "Search..."}
    />
  );
}
