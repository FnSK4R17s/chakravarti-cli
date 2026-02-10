/**
 * @module SpecWorkflow
 * @description
 * Workflow control panel for managing spec-to-implementation progression. Provides
 * step-by-step buttons for clarification resolution, design generation, and task
 * generation with visual progress indicators.
 *
 * @context
 * Displayed in SpecEditor sidebar. Shows current workflow stage and enables
 * triggering next steps. Each step becomes available after previous completes.
 *
 * @dependencies
 * - useValidateSpec, useGenerateDesign, useGenerateTasks: Hooks for workflow actions
 * - toast: Sonner toast notifications for feedback
 * - shadcn/ui components: Button, Badge for consistent UI
 *
 * @example
 * <SpecWorkflow
 *   specName="my-feature"
 *   unresolvedClarifications={2}
 *   hasDesign={false}
 *   hasTasks={false}
 *   onClarifyClick={openClarifyModal}
 * />
 */

// === IMPORTS ===
import { useState } from 'react';
import { Button } from './ui/button';
import { Badge } from './ui/badge';
import { Loader2, CheckCircle2, AlertCircle, FileText, ListTodo, Lightbulb, Sparkles } from 'lucide-react';
import { useValidateSpec, useGenerateDesign, useGenerateTasks } from '../hooks/useSpec';
import { toast } from 'sonner';

// ============================================================
// TYPES
// ============================================================

/**
 * Props for SpecWorkflow component.
 * Workflow control panel for managing spec-to-implementation progression.
 */
interface SpecWorkflowProps {
    /** Name of the spec being managed */
    specName: string;
    /** Count of unresolved clarification questions */
    unresolvedClarifications?: number;
    /** Whether design.md has been generated */
    hasDesign?: boolean;
    /** Whether tasks.yaml has been generated */
    hasTasks?: boolean;
    /** Callback to open clarification resolution dialog */
    onClarifyClick?: () => void;
    /** Callback fired when design generation completes */
    onDesignComplete?: () => void;
    /** Callback fired when tasks generation completes */
    onTasksComplete?: () => void;
    /** Callback fired when validation completes with result */
    onValidationComplete?: (valid: boolean) => void;
}

export function SpecWorkflow({
    specName,
    unresolvedClarifications = 0,
    hasDesign = false,
    hasTasks = false,
    onClarifyClick,
    onDesignComplete,
    onTasksComplete,
    onValidationComplete,
}: SpecWorkflowProps) {
    // === STATE ===
    /** Result from spec validation (valid flag and errors list) */
    const [validationResult, setValidationResult] = useState<{ valid: boolean; errors: string[] } | null>(null);
    /** Track if an async operation is in progress */
    const [isProcessing, setIsProcessing] = useState(false);
    /** Error message from failed operations */
    const [error, setError] = useState<string | null>(null);

    const validateMutation = useValidateSpec();
    const designMutation = useGenerateDesign();
    const tasksMutation = useGenerateTasks();

    // ============================================================
    // HANDLERS
    // ============================================================

    const handleValidate = async () => {
        setIsProcessing(true);
        setError(null);
        try {
            const result = await validateMutation.mutateAsync(specName);
            setValidationResult({
                valid: result.valid,
                errors: result.errors.map(e => `${e.field}: ${e.message}`),
            });
            onValidationComplete?.(result.valid);
        } catch (e) {
            setError(e instanceof Error ? e.message : 'Validation failed');
        } finally {
            setIsProcessing(false);
        }
    };

    const handleDesign = async () => {
        setIsProcessing(true);
        setError(null);
        try {
            await designMutation.mutateAsync(specName);
            toast.success('Design Generated', {
                description: 'design.md has been created successfully',
            });
            onDesignComplete?.();
        } catch (e) {
            const errorMsg = e instanceof Error ? e.message : 'Design generation failed';
            setError(errorMsg);
            toast.error('Design Generation Failed', {
                description: errorMsg,
            });
        } finally {
            setIsProcessing(false);
        }
    };

    const handleTasks = async () => {
        setIsProcessing(true);
        setError(null);
        try {
            await tasksMutation.mutateAsync(specName);
            toast.success('Tasks Generated', {
                description: 'tasks.yaml has been created successfully',
            });
            onTasksComplete?.();
        } catch (e) {
            const errorMsg = e instanceof Error ? e.message : 'Tasks generation failed';
            setError(errorMsg);
            toast.error('Tasks Generation Failed', {
                description: errorMsg,
            });
        } finally {
            setIsProcessing(false);
        }
    };

    // Determine current phase
    const needsClarification = unresolvedClarifications > 0;
    const canDesign = !needsClarification && !hasDesign;
    const canGenerateTasks = hasDesign && !hasTasks;
    const isComplete = hasTasks;

    return (
        <div className="space-y-4 p-4 bg-card rounded-lg border border-border/50">
            <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-semibold text-foreground">Workflow</h3>
                {isComplete && (
                    <Badge className="bg-success/20 text-success border-success/30">
                        <CheckCircle2 className="w-3 h-3 mr-1" />
                        Ready
                    </Badge>
                )}
            </div>

            {/* Error Display */}
            {error && (
                <div className="p-3 rounded-lg bg-error/10 border border-error/30 text-error text-sm">
                    <AlertCircle className="w-4 h-4 inline mr-2" />
                    {error}
                </div>
            )}

            {/* Validation Result */}
            {validationResult && (
                <div className={`p-3 rounded-lg text-sm ${validationResult.valid
                    ? 'bg-success/10 border border-success/30 text-success'
                    : 'bg-warning/10 border border-warning/30 text-warning'
                    }`}>
                    {validationResult.valid ? (
                        <>
                            <CheckCircle2 className="w-4 h-4 inline mr-2" />
                            Spec is valid!
                        </>
                    ) : (
                        <>
                            <AlertCircle className="w-4 h-4 inline mr-2" />
                            Validation failed:
                            <ul className="mt-2 ml-6 list-disc">
                                {validationResult.errors.map((err, i) => (
                                    <li key={i}>{err}</li>
                                ))}
                            </ul>
                        </>
                    )}
                </div>
            )}

            {/* Workflow Steps */}
            <div className="space-y-3">
                {/* Step 1: Clarify */}
                <div className={`flex items-center justify-between p-3 rounded-lg border ${needsClarification
                    ? 'bg-warning/5 border-warning/30'
                    : 'bg-muted/30 border-border/30'
                    }`}>
                    <div className="flex items-center gap-3">
                        <div className={`w-8 h-8 rounded-full flex items-center justify-center ${needsClarification ? 'bg-warning/20' : 'bg-success/20'
                            }`}>
                            {needsClarification ? (
                                <Lightbulb className="w-4 h-4 text-warning" />
                            ) : (
                                <CheckCircle2 className="w-4 h-4 text-success" />
                            )}
                        </div>
                        <div>
                            <div className="font-medium text-foreground">1. Clarify</div>
                            <div className="text-xs text-muted-foreground">
                                {needsClarification
                                    ? `${unresolvedClarifications} question(s) need answers`
                                    : 'No clarifications needed'}
                            </div>
                        </div>
                    </div>
                    {needsClarification && (
                        <Button
                            size="sm"
                            variant="outline"
                            onClick={onClarifyClick}
                            className="border-warning/30 text-warning hover:bg-warning/10"
                        >
                            Resolve
                        </Button>
                    )}
                </div>

                {/* Step 2: Design */}
                <div className={`flex items-center justify-between p-3 rounded-lg border ${canDesign
                    ? 'bg-accent/5 border-accent/30'
                    : hasDesign
                        ? 'bg-muted/30 border-border/30'
                        : 'bg-muted/10 border-border/20 opacity-50'
                    }`}>
                    <div className="flex items-center gap-3">
                        <div className={`w-8 h-8 rounded-full flex items-center justify-center ${hasDesign ? 'bg-success/20' : canDesign ? 'bg-accent/20' : 'bg-muted/30'
                            }`}>
                            {hasDesign ? (
                                <CheckCircle2 className="w-4 h-4 text-success" />
                            ) : (
                                <FileText className="w-4 h-4 text-accent" />
                            )}
                        </div>
                        <div>
                            <div className="font-medium text-foreground">2. Design</div>
                            <div className="text-xs text-muted-foreground">
                                {hasDesign ? 'design.md generated' : 'Generate technical design'}
                            </div>
                        </div>
                    </div>
                    {canDesign && (
                        <Button
                            size="sm"
                            onClick={handleDesign}
                            disabled={isProcessing}
                            className="bg-accent text-accent-foreground hover:bg-accent/90"
                        >
                            {isProcessing ? (
                                <>
                                    <Loader2 className="w-3 h-3 mr-1 animate-spin" />
                                    Generating...
                                </>
                            ) : (
                                <>
                                    <Sparkles className="w-3 h-3 mr-1" />
                                    Generate
                                </>
                            )}
                        </Button>
                    )}
                </div>

                {/* Step 3: Tasks */}
                <div className={`flex items-center justify-between p-3 rounded-lg border ${canGenerateTasks
                    ? 'bg-accent/5 border-accent/30'
                    : hasTasks
                        ? 'bg-muted/30 border-border/30'
                        : 'bg-muted/10 border-border/20 opacity-50'
                    }`}>
                    <div className="flex items-center gap-3">
                        <div className={`w-8 h-8 rounded-full flex items-center justify-center ${hasTasks ? 'bg-success/20' : canGenerateTasks ? 'bg-accent/20' : 'bg-muted/30'
                            }`}>
                            {hasTasks ? (
                                <CheckCircle2 className="w-4 h-4 text-success" />
                            ) : (
                                <ListTodo className="w-4 h-4 text-accent" />
                            )}
                        </div>
                        <div>
                            <div className="font-medium text-foreground">3. Tasks</div>
                            <div className="text-xs text-muted-foreground">
                                {hasTasks ? 'tasks.yaml generated' : 'Generate implementation tasks'}
                            </div>
                        </div>
                    </div>
                    {canGenerateTasks && (
                        <Button
                            size="sm"
                            onClick={handleTasks}
                            disabled={isProcessing}
                            className="bg-accent text-accent-foreground hover:bg-accent/90"
                        >
                            {isProcessing ? (
                                <>
                                    <Loader2 className="w-3 h-3 mr-1 animate-spin" />
                                    Generating...
                                </>
                            ) : (
                                <>
                                    <Sparkles className="w-3 h-3 mr-1" />
                                    Generate
                                </>
                            )}
                        </Button>
                    )}
                </div>
            </div>

            {/* Validate Button */}
            <div className="pt-2 border-t border-border/30">
                <Button
                    variant="outline"
                    size="sm"
                    onClick={handleValidate}
                    disabled={isProcessing}
                    className="w-full"
                >
                    {isProcessing ? (
                        <>
                            <Loader2 className="w-3 h-3 mr-2 animate-spin" />
                            Validating...
                        </>
                    ) : (
                        <>
                            <CheckCircle2 className="w-3 h-3 mr-2" />
                            Validate Spec
                        </>
                    )}
                </Button>
            </div>
        </div>
    );
}

export default SpecWorkflow;
