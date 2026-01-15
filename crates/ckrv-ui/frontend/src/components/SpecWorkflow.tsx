/**
 * SpecWorkflow - Workflow control component for spec management
 * Provides buttons for clarify, design, and tasks generation with progress indicators
 */
import { useState } from 'react';
import { Button } from './ui/button';
import { Badge } from './ui/badge';
import { Loader2, CheckCircle2, AlertCircle, FileText, ListTodo, Lightbulb, Sparkles } from 'lucide-react';
import { useValidateSpec, useGenerateDesign, useGenerateTasks } from '../hooks/useSpec';
import { toast } from 'sonner';

interface SpecWorkflowProps {
    specName: string;
    unresolvedClarifications?: number;
    hasDesign?: boolean;
    hasTasks?: boolean;
    onClarifyClick?: () => void;
    onDesignComplete?: () => void;
    onTasksComplete?: () => void;
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
    const [validationResult, setValidationResult] = useState<{ valid: boolean; errors: string[] } | null>(null);
    const [isProcessing, setIsProcessing] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const validateMutation = useValidateSpec();
    const designMutation = useGenerateDesign();
    const tasksMutation = useGenerateTasks();

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
                    <Badge className="bg-green-500/20 text-green-400 border-green-500/30">
                        <CheckCircle2 className="w-3 h-3 mr-1" />
                        Ready
                    </Badge>
                )}
            </div>

            {/* Error Display */}
            {error && (
                <div className="p-3 rounded-lg bg-red-500/10 border border-red-500/30 text-red-400 text-sm">
                    <AlertCircle className="w-4 h-4 inline mr-2" />
                    {error}
                </div>
            )}

            {/* Validation Result */}
            {validationResult && (
                <div className={`p-3 rounded-lg text-sm ${validationResult.valid
                    ? 'bg-green-500/10 border border-green-500/30 text-green-400'
                    : 'bg-yellow-500/10 border border-yellow-500/30 text-yellow-400'
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
                    ? 'bg-yellow-500/5 border-yellow-500/30'
                    : 'bg-muted/30 border-border/30'
                    }`}>
                    <div className="flex items-center gap-3">
                        <div className={`w-8 h-8 rounded-full flex items-center justify-center ${needsClarification ? 'bg-yellow-500/20' : 'bg-green-500/20'
                            }`}>
                            {needsClarification ? (
                                <Lightbulb className="w-4 h-4 text-yellow-400" />
                            ) : (
                                <CheckCircle2 className="w-4 h-4 text-green-400" />
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
                            className="border-yellow-500/30 text-yellow-400 hover:bg-yellow-500/10"
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
                        <div className={`w-8 h-8 rounded-full flex items-center justify-center ${hasDesign ? 'bg-green-500/20' : canDesign ? 'bg-accent/20' : 'bg-muted/30'
                            }`}>
                            {hasDesign ? (
                                <CheckCircle2 className="w-4 h-4 text-green-400" />
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
                        <div className={`w-8 h-8 rounded-full flex items-center justify-center ${hasTasks ? 'bg-green-500/20' : canGenerateTasks ? 'bg-accent/20' : 'bg-muted/30'
                            }`}>
                            {hasTasks ? (
                                <CheckCircle2 className="w-4 h-4 text-green-400" />
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
