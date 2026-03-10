/**
 * @module ClarifyModal
 * @description
 * Interactive dialog for answering spec clarification questions. Presents questions
 * one at a time with radio options and visual progress indicators. Supports navigation
 * between questions and batch submission.
 *
 * @context
 * Opened from SpecEditor when there are unresolved clarifications. Users answer
 * questions to refine the specification before task generation.
 *
 * @dependencies
 * - useClarifications: Hook for clarification data
 * - shadcn/ui components: Dialog, RadioGroup, Badge, Button for consistent UI
 *
 * @example
 * <ClarifyModal
 *   open={showModal}
 *   onOpenChange={setShowModal}
 *   specName="my-feature"
 *   clarifications={unresolvedClarifications}
 *   onSubmit={handleSubmitAnswers}
 * />
 */

// === IMPORTS ===
import { useState } from 'react';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
} from './ui/dialog';
import { Button } from './ui/button';
import { RadioGroup, RadioGroupItem } from './ui/radio-group';
import { Label } from './ui/label';
import { Badge } from './ui/badge';
import { Loader2, ChevronLeft, ChevronRight, HelpCircle, CheckCircle2 } from 'lucide-react';
import { type Clarification } from '../hooks/useSpec';

// ============================================================
// TYPES
// ============================================================

/**
 * Props for the ClarifyModal component.
 */
interface ClarifyModalProps {
    /** Whether the dialog is currently open */
    open: boolean;
    /** Callback fired when dialog open state changes */
    onOpenChange: (open: boolean) => void;
    /** Name of the spec being clarified */
    specName: string;
    /** Array of clarification questions to present */
    clarifications: Clarification[];
    /** Async callback fired when user submits all answers */
    onSubmit: (answers: { topic: string; answer: string }[]) => Promise<void>;
    /**
     * Whether the submission is in progress.
     * @default false
     */
    isSubmitting?: boolean;
}

export function ClarifyModal({
    open,
    onOpenChange,
    specName,
    clarifications,
    onSubmit,
    isSubmitting = false,
}: ClarifyModalProps) {
    // ============================================================
    // STATE
    // ============================================================

    /** Index of the currently displayed clarification question */
    const [currentIndex, setCurrentIndex] = useState(0);
    /** Map of clarification topics to their selected answers */
    const [answers, setAnswers] = useState<Record<string, string>>({});

    // Filter to only unresolved clarifications
    const unresolved = clarifications.filter(c => !c.resolved);

    if (unresolved.length === 0) {
        return null;
    }

    const current = unresolved[currentIndex];
    const isFirst = currentIndex === 0;
    const isLast = currentIndex === unresolved.length - 1;
    const hasAnswer = current && answers[current.topic];
    const allAnswered = unresolved.every(c => answers[c.topic]);

    // ============================================================
    // HANDLERS
    // ============================================================

    const handleAnswer = (answer: string) => {
        if (!current) return;
        setAnswers(prev => ({ ...prev, [current.topic]: answer }));
    };

    const handlePrev = () => {
        if (!isFirst) setCurrentIndex(prev => prev - 1);
    };

    const handleNext = () => {
        if (!isLast) setCurrentIndex(prev => prev + 1);
    };

    const handleSubmit = async () => {
        const answerList = Object.entries(answers).map(([topic, answer]) => ({
            topic,
            answer,
        }));
        await onSubmit(answerList);
        onOpenChange(false);
        setAnswers({});
        setCurrentIndex(0);
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-[550px] bg-card border-border flex flex-col !gap-0 p-0">
                {/* Header - fixed */}
                <DialogHeader className="shrink-0 px-5 pt-5 pb-3">
                    <div className="flex items-center justify-between">
                        <DialogTitle className="flex items-center gap-2 text-foreground text-base">
                            <HelpCircle className="w-4 h-4 text-warning" />
                            Clarification Needed
                        </DialogTitle>
                        <Badge variant="outline" className="text-muted-foreground text-xs">
                            {currentIndex + 1} / {unresolved.length}
                        </Badge>
                    </div>
                    <DialogDescription className="text-muted-foreground text-xs">
                        Spec: <span className="text-foreground font-medium">{specName}</span>
                    </DialogDescription>
                </DialogHeader>

                {/* Scrollable content - question + options */}
                {current && (
                    <div className="flex-1 min-h-0 overflow-y-auto px-5 py-3 space-y-3">
                        {/* Topic + Question */}
                        <div className="flex items-center gap-2">
                            <Badge className="bg-accent/20 text-accent border-accent/30 text-xs">
                                {current.topic}
                            </Badge>
                        </div>
                        <p className="text-sm font-medium text-foreground leading-relaxed">
                            {current.question}
                        </p>

                        {/* Options - compact */}
                        <RadioGroup
                            value={answers[current.topic] || ''}
                            onValueChange={handleAnswer}
                            className="space-y-2"
                        >
                            {current.options.map((option, idx) => {
                                const isSelected = answers[current.topic] === option.answer;
                                return (
                                    <div
                                        key={idx}
                                        className={`flex items-start space-x-2.5 px-3 py-2.5 rounded-md border transition-colors cursor-pointer ${isSelected
                                            ? 'bg-accent/10 border-accent/50'
                                            : 'bg-muted/20 border-border/50 hover:border-border'
                                            }`}
                                        onClick={() => handleAnswer(option.answer)}
                                    >
                                        <RadioGroupItem
                                            value={option.answer}
                                            id={`option-${idx}`}
                                            className="mt-0.5"
                                        />
                                        <div className="flex-1 min-w-0">
                                            <Label
                                                htmlFor={`option-${idx}`}
                                                className="text-sm font-medium text-foreground cursor-pointer"
                                            >
                                                <span className="inline-flex items-center justify-center w-5 h-5 rounded-full bg-muted text-muted-foreground text-xs mr-1.5">
                                                    {String.fromCharCode(65 + idx)}
                                                </span>
                                                {option.answer}
                                            </Label>
                                            {option.implications && (
                                                <p className="mt-0.5 text-xs text-muted-foreground ml-6.5">
                                                    → {option.implications}
                                                </p>
                                            )}
                                        </div>
                                        {isSelected && (
                                            <CheckCircle2 className="w-4 h-4 text-accent mt-0.5 shrink-0" />
                                        )}
                                    </div>
                                );
                            })}
                        </RadioGroup>
                    </div>
                )}

                {/* Footer - fixed, always visible */}
                <div className="shrink-0 px-5 pb-4 pt-3 border-t border-border space-y-2">
                    {/* Primary action */}
                    {isLast ? (
                        <Button
                            onClick={handleSubmit}
                            disabled={!allAnswered || isSubmitting}
                            className="w-full bg-accent text-accent-foreground hover:bg-accent/90"
                        >
                            {isSubmitting ? (
                                <>
                                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                                    Saving...
                                </>
                            ) : (
                                <>
                                    <CheckCircle2 className="w-4 h-4 mr-2" />
                                    Save All Answers
                                </>
                            )}
                        </Button>
                    ) : (
                        <Button
                            onClick={handleNext}
                            disabled={!hasAnswer}
                            className="w-full bg-accent text-accent-foreground hover:bg-accent/90"
                        >
                            Next Question
                            <ChevronRight className="w-4 h-4 ml-1" />
                        </Button>
                    )}

                    {/* Secondary actions + progress dots */}
                    <div className="flex items-center justify-between w-full">
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={handlePrev}
                            disabled={isFirst}
                        >
                            <ChevronLeft className="w-4 h-4 mr-1" />
                            Prev
                        </Button>

                        {/* Progress dots - inline */}
                        <div className="flex gap-1">
                            {unresolved.map((_, idx) => (
                                <div
                                    key={idx}
                                    className={`w-1.5 h-1.5 rounded-full transition-colors ${idx === currentIndex
                                        ? 'bg-accent'
                                        : answers[unresolved[idx].topic]
                                            ? 'bg-success'
                                            : 'bg-muted'
                                        }`}
                                />
                            ))}
                        </div>

                        <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => onOpenChange(false)}
                        >
                            Cancel
                        </Button>
                    </div>
                </div>
            </DialogContent>
        </Dialog>
    );
}

export default ClarifyModal;
