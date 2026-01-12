# Component Contracts: shadcn/ui Migration

**Feature**: 009-shadcn-ui-migration  
**Date**: 2026-01-11

## Overview

This document defines the interface contracts for shadcn/ui components as they will be used in this project. Since this is a frontend-only migration with no API changes, contracts focus on component props and variant specifications.

---

## Button Contract

```typescript
interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "default" | "destructive" | "outline" | "secondary" | "ghost" | "link"
  size?: "default" | "sm" | "lg" | "icon"
  asChild?: boolean
  loading?: boolean        // Custom extension
  loadingText?: string     // Custom extension
}
```

### Usage Examples
```tsx
<Button variant="default">Primary Action</Button>
<Button variant="destructive">Delete</Button>
<Button variant="ghost" size="icon"><XIcon /></Button>
<Button loading loadingText="Saving...">Save</Button>
```

---

## Card Contract

```typescript
interface CardProps extends React.HTMLAttributes<HTMLDivElement> {}

interface CardHeaderProps extends React.HTMLAttributes<HTMLDivElement> {}

interface CardTitleProps extends React.HTMLAttributes<HTMLHeadingElement> {}

interface CardDescriptionProps extends React.HTMLAttributes<HTMLParagraphElement> {}

interface CardContentProps extends React.HTMLAttributes<HTMLDivElement> {}

interface CardFooterProps extends React.HTMLAttributes<HTMLDivElement> {}
```

### Usage Example
```tsx
<Card>
  <CardHeader>
    <CardTitle>Agent Configuration</CardTitle>
    <CardDescription>Configure your AI agent</CardDescription>
  </CardHeader>
  <CardContent>
    {/* Form fields */}
  </CardContent>
  <CardFooter>
    <Button>Save</Button>
  </CardFooter>
</Card>
```

---

## Badge Contract

```typescript
interface BadgeProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: "default" | "secondary" | "destructive" | "outline" 
    | "success" | "warning" | "info"  // Custom variants
}
```

### Custom Variant Definitions
```css
/* Add to badge component */
.badge-success { background: var(--accent-green-dim); color: var(--accent-green); }
.badge-warning { background: var(--accent-amber-dim); color: var(--accent-amber); }
.badge-info { background: var(--accent-purple-dim); color: var(--accent-purple); }
```

### Usage Examples
```tsx
<Badge variant="default">Active</Badge>
<Badge variant="success">Completed</Badge>
<Badge variant="destructive">Failed</Badge>
<Badge variant="warning">Pending</Badge>
```

---

## Dialog Contract

```typescript
interface DialogProps {
  open?: boolean
  onOpenChange?: (open: boolean) => void
  modal?: boolean
  children: React.ReactNode
}

interface DialogTriggerProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  asChild?: boolean
}

interface DialogContentProps extends React.HTMLAttributes<HTMLDivElement> {
  onEscapeKeyDown?: (event: KeyboardEvent) => void
  onPointerDownOutside?: (event: PointerDownOutsideEvent) => void
}

interface DialogHeaderProps extends React.HTMLAttributes<HTMLDivElement> {}
interface DialogFooterProps extends React.HTMLAttributes<HTMLDivElement> {}
interface DialogTitleProps extends React.HTMLAttributes<HTMLHeadingElement> {}
interface DialogDescriptionProps extends React.HTMLAttributes<HTMLParagraphElement> {}
```

### Usage Example
```tsx
<Dialog open={isOpen} onOpenChange={setIsOpen}>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Configure Agent</DialogTitle>
      <DialogDescription>Set up your AI agent configuration</DialogDescription>
    </DialogHeader>
    <div className="grid gap-4 py-4">
      {/* Form content */}
    </div>
    <DialogFooter>
      <Button variant="secondary" onClick={() => setIsOpen(false)}>Cancel</Button>
      <Button onClick={handleSave}>Save</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
```

---

## Input Contract

```typescript
interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {}
```

### Usage Example
```tsx
<Input 
  type="text" 
  placeholder="Agent name" 
  value={name}
  onChange={(e) => setName(e.target.value)}
/>
```

---

## Select Contract

```typescript
interface SelectProps {
  value?: string
  onValueChange?: (value: string) => void
  defaultValue?: string
  disabled?: boolean
  children: React.ReactNode
}

interface SelectTriggerProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  children: React.ReactNode
}

interface SelectContentProps extends React.HTMLAttributes<HTMLDivElement> {
  position?: "popper" | "item-aligned"
  side?: "top" | "right" | "bottom" | "left"
}

interface SelectItemProps extends React.HTMLAttributes<HTMLDivElement> {
  value: string
  disabled?: boolean
}
```

### Usage Example
```tsx
<Select value={agentType} onValueChange={setAgentType}>
  <SelectTrigger>
    <SelectValue placeholder="Select agent type" />
  </SelectTrigger>
  <SelectContent>
    <SelectItem value="claude">Claude Code</SelectItem>
    <SelectItem value="gemini">Gemini CLI</SelectItem>
    <SelectItem value="openrouter">OpenRouter</SelectItem>
  </SelectContent>
</Select>
```

---

## Tabs Contract

```typescript
interface TabsProps extends React.HTMLAttributes<HTMLDivElement> {
  value?: string
  onValueChange?: (value: string) => void
  defaultValue?: string
}

interface TabsListProps extends React.HTMLAttributes<HTMLDivElement> {}

interface TabsTriggerProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  value: string
  disabled?: boolean
}

interface TabsContentProps extends React.HTMLAttributes<HTMLDivElement> {
  value: string
  forceMount?: boolean
}
```

### Usage Example
```tsx
<Tabs value={view} onValueChange={setView}>
  <TabsList>
    <TabsTrigger value="cards"><LayoutGrid size={16} /> Cards</TabsTrigger>
    <TabsTrigger value="list"><List size={16} /> List</TabsTrigger>
    <TabsTrigger value="yaml"><Code size={16} /> YAML</TabsTrigger>
  </TabsList>
  <TabsContent value="cards">{/* Cards view */}</TabsContent>
  <TabsContent value="list">{/* List view */}</TabsContent>
  <TabsContent value="yaml">{/* YAML view */}</TabsContent>
</Tabs>
```

---

## Tooltip Contract

```typescript
interface TooltipProviderProps {
  delayDuration?: number
  skipDelayDuration?: number
  disableHoverableContent?: boolean
  children: React.ReactNode
}

interface TooltipProps {
  open?: boolean
  onOpenChange?: (open: boolean) => void
  defaultOpen?: boolean
  delayDuration?: number
  children: React.ReactNode
}

interface TooltipTriggerProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  asChild?: boolean
}

interface TooltipContentProps extends React.HTMLAttributes<HTMLDivElement> {
  side?: "top" | "right" | "bottom" | "left"
  sideOffset?: number
  align?: "start" | "center" | "end"
}
```

### Usage Example
```tsx
<TooltipProvider>
  <Tooltip>
    <TooltipTrigger asChild>
      <Button variant="ghost" size="icon"><Settings size={18} /></Button>
    </TooltipTrigger>
    <TooltipContent>
      <p>Settings</p>
    </TooltipContent>
  </Tooltip>
</TooltipProvider>
```

---

## Collapsible Contract

```typescript
interface CollapsibleProps extends React.HTMLAttributes<HTMLDivElement> {
  open?: boolean
  onOpenChange?: (open: boolean) => void
  defaultOpen?: boolean
  disabled?: boolean
}

interface CollapsibleTriggerProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  asChild?: boolean
}

interface CollapsibleContentProps extends React.HTMLAttributes<HTMLDivElement> {
  forceMount?: boolean
}
```

### Usage Example
```tsx
<Collapsible open={isOpen} onOpenChange={setIsOpen}>
  <CollapsibleTrigger asChild>
    <Button variant="ghost">
      {isOpen ? <ChevronDown /> : <ChevronRight />}
      User Stories ({count})
    </Button>
  </CollapsibleTrigger>
  <CollapsibleContent>
    {/* Section content */}
  </CollapsibleContent>
</Collapsible>
```

---

## ScrollArea Contract

```typescript
interface ScrollAreaProps extends React.HTMLAttributes<HTMLDivElement> {
  type?: "auto" | "always" | "scroll" | "hover"
  scrollHideDelay?: number
}

interface ScrollBarProps extends React.HTMLAttributes<HTMLDivElement> {
  orientation?: "vertical" | "horizontal"
}
```

### Usage Example
```tsx
<ScrollArea className="h-[400px]">
  {logs.map((log, i) => (
    <LogLine key={i} log={log} />
  ))}
</ScrollArea>
```

---

## Progress Contract

```typescript
interface ProgressProps extends React.HTMLAttributes<HTMLDivElement> {
  value?: number
  max?: number
}
```

### Usage Example
```tsx
<Progress value={completedTasks / totalTasks * 100} />
```

---

## Skeleton Contract

```typescript
interface SkeletonProps extends React.HTMLAttributes<HTMLDivElement> {}
```

### Usage Example
```tsx
<Card>
  <CardHeader>
    <Skeleton className="h-4 w-[250px]" />
    <Skeleton className="h-4 w-[200px]" />
  </CardHeader>
  <CardContent>
    <Skeleton className="h-[200px] w-full" />
  </CardContent>
</Card>
```

---

## Alert Contract

```typescript
interface AlertProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: "default" | "destructive"
}

interface AlertTitleProps extends React.HTMLAttributes<HTMLHeadingElement> {}
interface AlertDescriptionProps extends React.HTMLAttributes<HTMLParagraphElement> {}
```

### Usage Example
```tsx
<Alert variant="destructive">
  <AlertCircle className="h-4 w-4" />
  <AlertTitle>Error</AlertTitle>
  <AlertDescription>
    Something went wrong during execution.
  </AlertDescription>
</Alert>
```

---

## Separator Contract

```typescript
interface SeparatorProps extends React.HTMLAttributes<HTMLDivElement> {
  orientation?: "horizontal" | "vertical"
  decorative?: boolean
}
```

### Usage Example
```tsx
<Separator className="my-4" />
```

---

## Custom Extensions

### LoadingButton (extends Button)

Since shadcn/ui Button doesn't have built-in loading state, we'll extend it:

```typescript
interface LoadingButtonProps extends ButtonProps {
  loading?: boolean
  loadingText?: string
}

const LoadingButton: React.FC<LoadingButtonProps> = ({
  loading,
  loadingText,
  children,
  disabled,
  ...props
}) => {
  return (
    <Button disabled={loading || disabled} {...props}>
      {loading ? (
        <>
          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          {loadingText || children}
        </>
      ) : (
        children
      )}
    </Button>
  )
}
```

---

## Migration Checklist

Each component migration should verify:

- [ ] Props interface matches contract
- [ ] All variants render correctly
- [ ] Focus states visible (keyboard navigation)
- [ ] Dark theme colors correct
- [ ] Animations smooth (enter/exit)
- [ ] Screen reader accessible
- [ ] Existing functionality preserved
