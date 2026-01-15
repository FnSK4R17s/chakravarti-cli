/**
 * NewSpecDialog - Dialog for creating new specs from natural language description
 */
import { useState } from 'react';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
    DialogFooter,
} from './ui/dialog';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Label } from './ui/label';
import { Textarea } from './ui/textarea';
import { Loader2, Sparkles, FileText } from 'lucide-react';
import { useCreateSpec } from '../hooks/useSpec';

interface NewSpecDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onSuccess?: (specId: string) => void;
}

export function NewSpecDialog({
    open,
    onOpenChange,
    onSuccess,
}: NewSpecDialogProps) {
    const [description, setDescription] = useState('');
    const [name, setName] = useState('');
    const [error, setError] = useState<string | null>(null);

    const createSpec = useCreateSpec();

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();

        if (!description.trim()) {
            setError('Please enter a feature description');
            return;
        }

        setError(null);

        try {
            const result = await createSpec.mutateAsync({
                description: description.trim(),
                name: name.trim() || undefined,
            });

            if (result.success && result.spec_id) {
                onOpenChange(false);
                setDescription('');
                setName('');
                onSuccess?.(result.spec_id);
            } else if (result.error) {
                setError(result.error);
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to create spec');
        }
    };

    const isSubmitting = createSpec.isPending;

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-[550px] bg-card border-border">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2 text-foreground">
                        <Sparkles className="w-5 h-5 text-accent" />
                        Create New Specification
                    </DialogTitle>
                    <DialogDescription className="text-muted-foreground">
                        Describe your feature in natural language. AI will generate a comprehensive specification.
                    </DialogDescription>
                </DialogHeader>

                <form onSubmit={handleSubmit} className="space-y-4 py-4">
                    {/* Description */}
                    <div className="space-y-2">
                        <Label htmlFor="description" className="text-foreground">
                            Feature Description <span className="text-red-400">*</span>
                        </Label>
                        <Textarea
                            id="description"
                            value={description}
                            onChange={(e) => setDescription(e.target.value)}
                            placeholder="e.g., Add user authentication with email/password login, password reset, and social OAuth"
                            className="min-h-[120px] bg-background border-border text-foreground placeholder:text-muted-foreground resize-none"
                            disabled={isSubmitting}
                        />
                        <p className="text-xs text-muted-foreground">
                            Be specific about what you want to build. The more detail, the better the spec.
                        </p>
                    </div>

                    {/* Optional Name */}
                    <div className="space-y-2">
                        <Label htmlFor="name" className="text-foreground">
                            Short Name <span className="text-muted-foreground">(optional)</span>
                        </Label>
                        <Input
                            id="name"
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                            placeholder="e.g., user-auth"
                            className="bg-background border-border text-foreground placeholder:text-muted-foreground"
                            disabled={isSubmitting}
                        />
                        <p className="text-xs text-muted-foreground">
                            If not provided, a name will be auto-generated from the description.
                        </p>
                    </div>

                    {/* Error Display */}
                    {error && (
                        <div className="p-3 rounded-lg bg-red-500/10 border border-red-500/30 text-red-400 text-sm">
                            {error}
                        </div>
                    )}

                    {/* Info box */}
                    <div className="p-4 rounded-lg bg-muted/30 border border-border/50">
                        <div className="flex items-start gap-3">
                            <FileText className="w-5 h-5 text-muted-foreground mt-0.5" />
                            <div className="text-sm text-muted-foreground">
                                <p className="font-medium text-foreground mb-1">What happens next?</p>
                                <ul className="space-y-1">
                                    <li>• AI generates a rich spec.yaml with user stories</li>
                                    <li>• You may need to answer clarification questions</li>
                                    <li>• Then generate design and implementation tasks</li>
                                </ul>
                            </div>
                        </div>
                    </div>
                </form>

                <DialogFooter>
                    <Button
                        variant="ghost"
                        onClick={() => onOpenChange(false)}
                        disabled={isSubmitting}
                    >
                        Cancel
                    </Button>
                    <Button
                        onClick={handleSubmit}
                        disabled={!description.trim() || isSubmitting}
                        className="bg-accent text-accent-foreground hover:bg-accent/90"
                    >
                        {isSubmitting ? (
                            <>
                                <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                                Generating...
                            </>
                        ) : (
                            <>
                                <Sparkles className="w-4 h-4 mr-2" />
                                Generate Spec
                            </>
                        )}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}

export default NewSpecDialog;
