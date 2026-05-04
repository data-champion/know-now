# Accessibility — WCAG 2.2 AA Compliance

## Automated Audits

axe-core runs in CI via vitest (`src/a11y.test.tsx`). Serious and critical violations block merge.

## Keyboard Walkthrough

### Navigation

1. **Skip link**: Tab on page load reveals "Skip to main content" link.
2. **Tab navigation**: All tabs in the top nav bar are focusable and activated with Enter/Space.
3. **Focus management**: On tab change, focus moves to `<main>` element.

### Entity List

1. Tab into the search input — type to filter.
2. Tab to domain filter dropdown — use arrow keys to select.
3. Tab into the entity list items — each item is focusable.
4. Press Enter or Space on an entity to select it and open the detail view.

### Entity Detail

1. Detail panel appears to the right of the entity list.
2. Close button has `aria-label="Close detail view"` and is keyboard-accessible.
3. Attribute table uses `scope="col"` for column headers.

### Relationship Graph

1. Tab to the Graph/Table toggle — switch with radio buttons.
2. **Graph view**: Tab to the SVG canvas. Use Arrow keys to cycle through nodes. Enter/Space selects a node.
3. **Table view**: Standard table navigation. All headers have `scope="col"`.

### Generation / Manifest / Health Views

1. All definition lists (`dl`) are properly structured with `dt`/`dd` pairs.
2. Collapsible sections use native `<details>/<summary>` elements.

### Review Summary

1. Sub-tab navigation (Summary / Open Questions / Change Approval) is keyboard accessible.
2. Export and copy buttons are focusable with visible focus indicators.
3. Status dropdowns in the approval table have `aria-label` attributes.

### Open Questions Register

1. Priority filter dropdown is keyboard-accessible.
2. Table uses proper header scoping.

### Traceability View

1. Entity/artifact list items are focusable with `role="option"` and `aria-selected`.
2. Enter/Space selects an item. Detail pane updates with `aria-live="polite"`.

## Screen Reader Notes

### NVDA (Windows)

- All headings are navigable via H key.
- Tables announce row/column counts.
- `aria-label` on SVG graph announces "Entity relationship graph".
- `aria-live="polite"` regions announce filter results and detail changes.

### VoiceOver (macOS)

- Rotor lists all headings, tables, and form controls.
- Graph SVG is announced as an image with descriptive label.
- Skip link works with VO+Tab.

## WCAG 2.2 AA Checklist

| Criterion | Status | Notes |
|-----------|--------|-------|
| 1.1.1 Non-text Content | Pass | Graph has aria-label; SVG nodes have aria-label |
| 1.3.1 Info and Relationships | Pass | Semantic HTML (headings, tables, lists, dl) |
| 1.3.2 Meaningful Sequence | Pass | DOM order matches visual order |
| 1.4.3 Contrast (Minimum) | Pass | Dark theme uses high-contrast palette |
| 1.4.11 Non-text Contrast | Pass | Focus indicators use 2px accent color outline |
| 2.1.1 Keyboard | Pass | All interactive elements are keyboard-accessible |
| 2.1.2 No Keyboard Trap | Pass | Tab cycles through all elements without trapping |
| 2.4.1 Bypass Blocks | Pass | Skip-to-content link |
| 2.4.2 Page Titled | Pass | "know-now dashboard" in document title |
| 2.4.3 Focus Order | Pass | Logical tab order follows visual layout |
| 2.4.6 Headings and Labels | Pass | All sections have descriptive headings |
| 2.4.7 Focus Visible | Pass | focus-visible styles on all interactive elements |
| 3.1.1 Language of Page | Pass | `<html lang="en">` |
| 4.1.2 Name, Role, Value | Pass | ARIA roles and labels on custom controls |
