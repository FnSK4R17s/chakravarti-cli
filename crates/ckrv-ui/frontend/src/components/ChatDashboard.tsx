/**
 * @module ChatDashboard
 * @description
 * Main landing page for the chakravarti UI providing a chat-like interface for
 * creating new specifications. Shows existing spec status and guides users through
 * the workflow with contextual actions.
 *
 * @context
 * Rendered as the default/home page of the dashboard. Users describe features to
 * create specs, or navigate to existing spec workflows. Adapts UI based on whether
 * specs exist and their implementation status.
 *
 * @dependencies
 * - useQuery/useMutation: React Query for status and spec operations
 * - useNavigation: Context for navigating between pages
 * - shadcn/ui components: Button for consistent UI
 *
 * @example
 * // Rendered directly as a page component
 * <ChatDashboard />
 *
 * // Provides spec creation with suggestion chips and workflow guidance
 */

// === IMPORTS ===
import React, { useState } from 'react';
import { useMutation, useQueryClient, useQuery } from '@tanstack/react-query';
import { Sparkles, Send, Loader2, Code2, Terminal, Globe, Wrench, FileCode, Zap, ArrowRight, Plus, TestTube2, CheckCircle2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { useNavigation } from '../App';

// ============================================================
// TYPES
// ============================================================

interface SystemStatus {
    is_ready: boolean;
}

interface SpecListItem {
    name: string;
    path: string;
    has_tasks: boolean;
    has_plan: boolean;
    has_implementation: boolean;
}

interface SpecsResponse {
    specs: SpecListItem[];
    count: number;
}

// ============================================================
// CONSTANTS
// ============================================================

const suggestionChips = [
    { icon: <Code2 size={14} />, label: 'REST API', prompt: 'Build a REST API with authentication and CRUD endpoints' },
    { icon: <Terminal size={14} />, label: 'CLI Tool', prompt: 'Create a command-line tool with argument parsing' },
    { icon: <Globe size={14} />, label: 'Web App', prompt: 'Build a web application with frontend and backend' },
    { icon: <Wrench size={14} />, label: 'Refactor', prompt: 'Refactor existing codebase for better maintainability' },
    { icon: <FileCode size={14} />, label: 'Add Feature', prompt: 'Add a new feature to the existing application' },
    { icon: <Zap size={14} />, label: 'Fix Bug', prompt: 'Debug and fix an issue in the codebase' },
];

// ============================================================
// MAIN COMPONENT
// ============================================================

export const ChatDashboard: React.FC = () => {
    const queryClient = useQueryClient();
    const { setCurrentPage } = useNavigation();
    // === State ===
    /** Feature description input text */
    const [description, setDescription] = useState('');
    /** Whether the new spec input form is visible */
    const [showNewSpecInput, setShowNewSpecInput] = useState(false);

    // Check if initialized
    const { data: status } = useQuery<SystemStatus>({
        queryKey: ['status'],
        queryFn: async () => {
            const res = await fetch('/api/status');
            return res.json();
        },
        refetchInterval: 5000,
    });

    // Fetch existing specs
    const { data: specsData } = useQuery<SpecsResponse>({
        queryKey: ['specs'],
        queryFn: async () => {
            const res = await fetch('/api/specs');
            return res.json();
        },
    });

    const isInitialized = status?.is_ready ?? false;
    const existingSpecs = specsData?.specs ?? [];
    const hasExistingSpec = existingSpecs.length > 0;
    const latestSpec = existingSpecs[existingSpecs.length - 1];

    // Create spec mutation
    const createSpecMutation = useMutation({
        mutationFn: async (params: { description: string }) => {
            const res = await fetch('/api/command/spec-new', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(params),
            });
            if (!res.ok) {
                const body = await res.json().catch(() => null);
                throw new Error(body?.error || `Spec creation failed (HTTP ${res.status})`);
            }
            return res.json();
        },
        onSuccess: () => {
            setDescription('');
            setShowNewSpecInput(false);
            queryClient.invalidateQueries({ queryKey: ['status'] });
            queryClient.invalidateQueries({ queryKey: ['specs'] });
        },
    });

    // === Handlers ===
    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (description.trim() && isInitialized) {
            createSpecMutation.mutate({ description: description.trim() });
        }
    };

    const handleChipClick = (prompt: string) => {
        setDescription(prompt);
    };

    // Show spec creation UI
    const showCreationUI = !hasExistingSpec || showNewSpecInput;

    return (
        <div className="h-full flex flex-col items-center justify-center px-4">
            <div className="w-full max-w-2xl">
                {/* Header */}
                <div className="text-center mb-8">
                    {/* Terminal-style Logo */}
                    <div className="inline-flex items-center gap-2 mb-6 px-4 py-2 bg-muted rounded-lg border border-border">
                        <span className="text-primary font-mono text-lg">$</span>
                        <span className="font-mono font-bold text-xl text-foreground">ckrv</span>
                        <span className="w-2 h-5 bg-primary animate-pulse" />
                    </div>
                    <h1 className="text-3xl font-bold text-foreground mb-2">
                        {latestSpec?.has_implementation
                            ? '🎉 Implementation Complete!'
                            : hasExistingSpec && !showNewSpecInput
                                ? 'Spec Ready'
                                : 'What would you like to build?'}
                    </h1>
                    <p className="text-muted-foreground">
                        {latestSpec?.has_implementation
                            ? 'Your code has been generated. Run tests to verify everything works!'
                            : hasExistingSpec && !showNewSpecInput
                                ? `Your spec "${latestSpec?.name}" is ready to work on`
                                : 'Describe your feature and AI will generate a specification'
                        }
                    </p>
                </div>

                {/* Existing Spec Actions */}
                {hasExistingSpec && !showNewSpecInput && (
                    <div className="space-y-4 mb-8">
                        {latestSpec?.has_implementation ? (
                            /* Implementation Complete - Guide to Tests */
                            <>
                                <div className="flex items-center justify-center gap-2 text-success mb-4">
                                    <CheckCircle2 size={20} />
                                    <span className="text-sm font-medium">All coding tasks completed</span>
                                </div>
                                <div className="flex flex-col sm:flex-row gap-3 justify-center">
                                    <Button
                                        onClick={() => setCurrentPage('test')}
                                        className="gap-2 bg-success text-success-foreground hover:bg-success/90"
                                    >
                                        <TestTube2 size={18} />
                                        Go to Tests Page
                                    </Button>
                                    <Button
                                        variant="outline"
                                        onClick={() => setShowNewSpecInput(true)}
                                        className="gap-2"
                                    >
                                        <Plus size={18} />
                                        Create New Spec
                                    </Button>
                                </div>
                                <p className="text-xs text-muted-foreground text-center mt-2">
                                    Run tests to verify your implementation, then check the QA page for final review
                                </p>
                            </>
                        ) : (
                            /* Spec Ready - Guide to Code */
                            <>
                                <div className="flex flex-col sm:flex-row gap-3 justify-center">
                                    <Button
                                        onClick={() => setCurrentPage('code')}
                                        className="gap-2 bg-gradient-to-r from-primary to-primary/70"
                                    >
                                        <ArrowRight size={18} />
                                        Go to Code Page
                                    </Button>
                                    <Button
                                        variant="outline"
                                        onClick={() => setShowNewSpecInput(true)}
                                        className="gap-2"
                                    >
                                        <Plus size={18} />
                                        Create New Spec
                                    </Button>
                                </div>
                            </>
                        )}
                        <p className="text-xs text-muted-foreground text-center">
                            {existingSpecs.length} spec{existingSpecs.length !== 1 ? 's' : ''} in this project
                        </p>
                    </div>
                )}

                {/* Chat Input - Show when no spec exists or user wants to create new */}
                {showCreationUI && (
                    <>
                        <form onSubmit={handleSubmit} className="mb-6">
                            <div
                                className={cn(
                                    "relative rounded-xl border border-border bg-muted/50 transition-all",
                                    "focus-within:border-primary focus-within:ring-1 focus-within:ring-primary/20",
                                    !isInitialized && "opacity-50"
                                )}
                            >
                                <textarea
                                    value={description}
                                    onChange={(e) => setDescription(e.target.value)}
                                    placeholder={isInitialized
                                        ? "Describe your feature... e.g., 'Add user authentication with OAuth2 support'"
                                        : "Initialize the project first (Settings → Initialize)"
                                    }
                                    disabled={!isInitialized || createSpecMutation.isPending}
                                    rows={3}
                                    className={cn(
                                        "w-full px-4 py-4 pr-14 bg-transparent text-foreground placeholder:text-muted-foreground",
                                        "resize-none border-0 outline-none text-sm leading-relaxed",
                                        "disabled:cursor-not-allowed"
                                    )}
                                    onKeyDown={(e) => {
                                        if (e.key === 'Enter' && !e.shiftKey) {
                                            e.preventDefault();
                                            handleSubmit(e);
                                        }
                                    }}
                                />

                                {/* Submit Button */}
                                <div className="absolute right-3 bottom-3">
                                    <Button
                                        type="submit"
                                        size="icon"
                                        disabled={!description.trim() || !isInitialized || createSpecMutation.isPending}
                                        className={cn(
                                            "h-9 w-9 rounded-lg transition-all",
                                            description.trim() && isInitialized
                                                ? "bg-primary hover:bg-primary/90"
                                                : "bg-muted-foreground/20"
                                        )}
                                    >
                                        {createSpecMutation.isPending ? (
                                            <Loader2 size={16} className="animate-spin" />
                                        ) : (
                                            <Send size={16} />
                                        )}
                                    </Button>
                                </div>
                            </div>

                            <p className="text-xs text-muted-foreground mt-2 text-center">
                                Press <kbd className="px-1.5 py-0.5 bg-muted rounded text-xs">Enter</kbd> to send, <kbd className="px-1.5 py-0.5 bg-muted rounded text-xs">Shift+Enter</kbd> for new line
                            </p>
                        </form>

                        {/* Suggestion Chips */}
                        <div className="flex flex-wrap justify-center gap-2">
                            {suggestionChips.map((chip) => (
                                <button
                                    key={chip.label}
                                    onClick={() => handleChipClick(chip.prompt)}
                                    disabled={!isInitialized}
                                    className={cn(
                                        "flex items-center gap-2 px-3 py-2 rounded-lg text-sm",
                                        "bg-muted/50 border border-border text-muted-foreground",
                                        "hover:bg-muted hover:text-foreground hover:border-muted-foreground/30",
                                        "transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                                    )}
                                >
                                    <Sparkles size={12} className="text-primary" />
                                    {chip.label}
                                </button>
                            ))}
                        </div>

                        {/* Not initialized warning */}
                        {!isInitialized && (
                            <div className="mt-8 text-center">
                                <p className="text-sm text-muted-foreground">
                                    Project not initialized. Go to <strong>Settings</strong> to initialize.
                                </p>
                            </div>
                        )}

                        {/* Success feedback */}
                        {createSpecMutation.isSuccess && (
                            <div className="mt-6 text-center">
                                <div className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-success/20 text-success text-sm">
                                    <Sparkles size={16} />
                                    Specification created! Check the Code page.
                                </div>
                            </div>
                        )}

                        {/* Error feedback */}
                        {createSpecMutation.isError && (
                            <div className="mt-6 text-center">
                                <div className="inline-flex items-center gap-2 px-4 py-3 rounded-lg bg-destructive/15 text-destructive text-sm max-w-xl">
                                    <span className="shrink-0">✕</span>
                                    <span className="text-left">{createSpecMutation.error.message}</span>
                                </div>
                            </div>
                        )}

                        {/* Back button when creating new spec */}
                        {showNewSpecInput && (
                            <div className="mt-4 text-center">
                                <button
                                    onClick={() => setShowNewSpecInput(false)}
                                    className="text-sm text-muted-foreground hover:text-foreground transition-colors"
                                >
                                    ← Back to existing specs
                                </button>
                            </div>
                        )}
                    </>
                )}
            </div>
        </div>
    );
};

export default ChatDashboard;
