# Feature Specification: CSS Theme Consolidation

**Feature Branch**: `001-css-theme-consolidation`  
**Created**: 2026-01-12  
**Status**: Draft  
**Input**: User description: "Find all disorganized CSS style references and put them in a convenient place so that I can quickly swap out themes using tweakcn. We need one big theme storage in index.css with Tailwind 4 and OKLCH format."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Theme Swapping with tweakcn (Priority: P1)

As a developer, I want all theme colors consolidated in one centralized location in `index.css` using OKLCH color format so that I can quickly swap themes using tweakcn without hunting through multiple component files.

**Why this priority**: This is the core value proposition - enabling rapid theme changes by having a single source of truth for all design tokens. Without this, theme swapping requires finding and replacing color values across dozens of files.

**Independent Test**: Can be fully tested by changing color values in the centralized theme section of `index.css` and verifying that all UI components update correctly without additional changes.

**Acceptance Scenarios**:

1. **Given** I want to update the primary accent color, **When** I modify a single CSS variable in `index.css`, **Then** all components using that accent color update automatically without requiring changes to component files.

2. **Given** I import a new theme from tweakcn, **When** I paste the OKLCH color values into the theme section, **Then** the entire application reflects the new theme without breaking any styles.

3. **Given** the theme uses OKLCH format, **When** I view the CSS variables, **Then** all color values follow the OKLCH syntax (e.g., `oklch(0.5 0.2 240)`).

---

### User Story 2 - Eliminate Inline CSS Variable References (Priority: P2)

As a developer, I want all component files to use Tailwind utility classes with semantic names (like `text-accent-cyan`) instead of inline `var(--accent-*)` references so that theme tokens are properly decoupled from component logic.

**Why this priority**: Inline CSS variable references in components create coupling that makes refactoring difficult and prevents Tailwind from properly processing styles. This must be done after centralization to ensure tokens are first properly defined.

**Independent Test**: Can be tested by searching component files for `var(--accent-*` and `var(--bg-*` patterns - none should exist after migration.

**Acceptance Scenarios**:

1. **Given** a component currently uses `text-[var(--accent-cyan)]`, **When** I migrate the component, **Then** it uses a semantic Tailwind class like `text-accent-cyan`.

2. **Given** I search all `.tsx` files for `var(--`, **When** the migration is complete, **Then** no arbitrary value syntax for theme colors exists in component files.

---

### User Story 3 - Tailwind 4 Native Theme Integration (Priority: P2)

As a developer, I want the theme to use Tailwind 4's native `@theme` directive so that colors are available as proper Tailwind utilities with autocomplete support in my IDE.

**Why this priority**: Tailwind 4's `@theme` directive enables native integration with the Tailwind compiler, providing better DX with autocomplete and type-safe class names. Equal priority with Story 2 as both improve developer experience.

**Independent Test**: Can be tested by using Tailwind classes in a component and verifying IDE autocomplete shows the custom theme colors.

**Acceptance Scenarios**:

1. **Given** I define colors in `@theme`, **When** I type `text-accent-` in a component, **Then** my IDE suggests `text-accent-cyan`, `text-accent-green`, etc.

2. **Given** the theme uses OKLCH format, **When** Tailwind 4 compiles the CSS, **Then** the generated output correctly includes OKLCH color values.

---

### User Story 4 - Light/Dark Theme Support (Priority: P3)

As a developer, I want the theme structure to support both light and dark modes via a `.dark` class or `@media (prefers-color-scheme)` so that I can add light mode support in the future without restructuring.

**Why this priority**: The current application is dark-mode only. Enabling light mode is not immediately required, but the theme architecture should support it without needing restructuring later.

**Independent Test**: Can be tested by toggling the `.dark` class on the root element and verifying that appropriate color variables are applied.

**Acceptance Scenarios**:

1. **Given** OKLCH colors are defined for dark mode, **When** I add a `.light` or `:root` (light) section, **Then** the same semantic tokens map to light-appropriate colors.

2. **Given** I toggle between `.dark` and no class on the root, **When** I inspect CSS variables, **Then** the variable values correctly reflect the active theme.

---

### Edge Cases

- What happens when a component uses a color that doesn't exist in the centralized theme?
  - Build should fail or warn (via Tailwind's safelist/purge config)
- How does system handle gradients that combine multiple theme colors?
  - Gradients should reference theme variables (e.g., `linear-gradient(to-r, oklch(var(--accent-cyan)), oklch(var(--accent-purple)))`)
- What happens if OKLCH colors aren't supported in older browsers?
  - Provide hex fallbacks within CSS using `@supports` or rely on PostCSS plugins

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: All color definitions MUST be centralized in a single `@theme` block in `index.css`
- **FR-002**: All colors MUST use OKLCH format for consistency with tweakcn and modern CSS standards
- **FR-003**: Component files MUST NOT contain inline `var(--accent-*)` or `var(--bg-*)` references with Tailwind arbitrary value syntax
- **FR-004**: The theme MUST expose semantic Tailwind utilities (e.g., `bg-accent-cyan`, `text-accent-green`, `border-accent-amber`)
- **FR-005**: The theme MUST support dark mode via the `.dark` class selector
- **FR-006**: The theme MUST retain all existing design tokens (accent colors, background colors, border colors, text colors)
- **FR-007**: Existing visual appearance MUST remain identical after migration (no visual regressions)
- **FR-008**: Theme MUST support "dim" variants of accent colors for subtle backgrounds (e.g., `bg-accent-cyan-dim`)
- **FR-009**: Animation timing tokens MUST remain as CSS variables (these are not colors and don't need OKLCH conversion)

### Key Entities

- **Theme Section**: The centralized `@theme` block in `index.css` containing all design tokens
- **Color Token**: A semantic name (e.g., `accent-cyan`) mapped to an OKLCH value
- **Variant Token**: A modifier of a base color (e.g., `accent-cyan-dim` for 15% opacity backgrounds)
- **Semantic Mapping**: The relationship between raw color tokens and shadcn/ui semantic variables (e.g., `--primary` → `accent-cyan`)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All ~150 inline CSS variable references in component files are migrated to Tailwind utility classes
- **SC-002**: Theme can be swapped by changing only the `index.css` file with zero component file modifications
- **SC-003**: Developer can import a tweakcn theme and apply it within 5 minutes
- **SC-004**: IDE autocomplete shows all custom theme colors when typing Tailwind classes
- **SC-005**: Visual diff between before/after migration shows zero unintended changes (verified via screenshot comparison or visual regression testing)
- **SC-006**: All existing colors are converted to OKLCH format

### Assumptions

- The project uses Tailwind CSS v4 with the `@theme` directive support
- The tweakcn tool outputs OKLCH-formatted color themes
- PostCSS or similar tooling handles any needed browser fallbacks for OKLCH
- The existing `.dark` class on the root element is the mechanism for dark mode
- Component files are all `.tsx` files in the `src/components` directory
