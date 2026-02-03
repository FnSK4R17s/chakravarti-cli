"use client"

import { Toaster as Sonner, toast } from "sonner"

type ToasterProps = React.ComponentProps<typeof Sonner>

const Toaster = ({ ...props }: ToasterProps) => {
    return (
        <Sonner
            theme="system"
            className="toaster group"
            toastOptions={{
                classNames: {
                    toast:
                        "group toast group-[.toaster]:bg-background group-[.toaster]:text-foreground group-[.toaster]:border-border group-[.toaster]:shadow-lg",
                    description: "group-[.toast]:text-muted-foreground",
                    actionButton:
                        "group-[.toast]:bg-primary group-[.toast]:text-primary-foreground",
                    cancelButton:
                        "group-[.toast]:bg-muted group-[.toast]:text-muted-foreground",
                    success: "group-[.toaster]:bg-success/10 group-[.toaster]:text-success group-[.toaster]:border-success/30",
                    error: "group-[.toaster]:bg-error/10 group-[.toaster]:text-error group-[.toaster]:border-error/30",
                    warning: "group-[.toaster]:bg-warning/10 group-[.toaster]:text-warning group-[.toaster]:border-warning/30",
                    info: "group-[.toaster]:bg-info/10 group-[.toaster]:text-info group-[.toaster]:border-info/30",
                },
            }}
            {...props}
        />
    )
}

export { Toaster, toast }
