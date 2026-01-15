/**
 * ClarifyModal - Interactive clarification dialog
 * Allows users to answer clarification questions for a spec
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
import { RadioGroup, RadioGroupItem } from './ui/radio-group';
import { Label } from './ui/label';
import { Badge } from './ui/badge';
import { Loader2, ChevronLeft, ChevronRight, HelpCircle, CheckCircle2 } from 'lucide-react';
import { type Clarification } from '../hooks/useSpec';

interface ClarifyModalProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    specName: string;
    clarifications: Clarification[];
    onSubmit: (answers: { topic: string; answer: string }[]) => Promise<void>;
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
    const [currentIndex, setCurrentIndex] = useState(0);
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
            <DialogContent className="sm:max-w-[600px] bg-card border-border">
                <DialogHeader>
                    <div className="flex items-center justify-between">
                        <DialogTitle className="flex items-center gap-2 text-foreground">
                            <HelpCircle className="w-5 h-5 text-yellow-400" />
                            Clarification Needed
                        </DialogTitle>
                        <Badge variant="outline" className="text-muted-foreground">
                            {currentIndex + 1} of {unresolved.length}
                        </Badge>
                    </div>
                    <DialogDescription className="text-muted-foreground">
                        Spec: <span className="text-foreground font-medium">{specName}</span>
                    </DialogDescription>
                </DialogHeader>

                {current && (
                    <div className="py-4 space-y-4">
                        {/* Topic Badge */}
                        <Badge className="bg-accent/20 text-accent border-accent/30">
                            {current.topic}
                        </Badge>

                        {/* Question */}
                        <p className="text-lg font-medium text-foreground leading-relaxed">
                            {current.question}
                        </p>

                        {/* Options */}
                        <RadioGroup
                            value={answers[current.topic] || ''}
                            onValueChange={handleAnswer}
                            className="space-y-3 pt-2"
                        >
                            {current.options.map((option, idx) => {
                                const isSelected = answers[current.topic] === option.answer;
                                return (
                                    <div
                                        key={idx}
                                        className={`flex items-start space-x-3 p-4 rounded-lg border transition-colors ${isSelected
                                                ? 'bg-accent/10 border-accent/50'
                                                : 'bg-muted/20 border-border/50 hover:border-border'
                                            }`}
                                    >
                                        <RadioGroupItem
                                            value={option.answer}
                                            id={`option-${idx}`}
                                            className="mt-1"
                                        />
                                        <div className="flex-1">
                                            <Label
                                                htmlFor={`option-${idx}`}
                                                className="text-base font-medium text-foreground cursor-pointer"
                                            >
                                                <span className="inline-flex items-center justify-center w-6 h-6 rounded-full bg-muted text-muted-foreground text-sm mr-2">
                                                    {String.fromCharCode(65 + idx)}
                                                </span>
                                                {option.answer}
                                            </Label>
                                            {option.implications && (
                                                <p className="mt-1 text-sm text-muted-foreground ml-8">
                                                    → {option.implications}
                                                </p>
                                            )}
                                        </div>
                                        {isSelected && (
                                            <CheckCircle2 className="w-5 h-5 text-accent mt-1" />
                                        )}
                                    </div>
                                );
                            })}
                        </RadioGroup>
                    </div>
                )}

                <DialogFooter className="flex items-center justify-between sm:justify-between">
                    <div className="flex gap-2">
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={handlePrev}
                            disabled={isFirst}
                        >
                            <ChevronLeft className="w-4 h-4 mr-1" />
                            Previous
                        </Button>
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={handleNext}
                            disabled={isLast || !hasAnswer}
                        >
                            Next
                            <ChevronRight className="w-4 h-4 ml-1" />
                        </Button>
                    </div>

                    <div className="flex gap-2">
                        <Button
                            variant="ghost"
                            onClick={() => onOpenChange(false)}
                        >
                            Cancel
                        </Button>
                        <Button
                            onClick={handleSubmit}
                            disabled={!allAnswered || isSubmitting}
                            className="bg-accent text-accent-foreground hover:bg-accent/90"
                        >
                            {isSubmitting ? (
                                <>
                                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                                    Saving...
                                </>
                            ) : (
                                <>
                                    <CheckCircle2 className="w-4 h-4 mr-2" />
                                    Save Answers ({Object.keys(answers).length})
                                </>
                            )}
                        </Button>
                    </div>
                </DialogFooter>

                {/* Progress dots */}
                <div className="flex justify-center gap-1 mt-2">
                    {unresolved.map((_, idx) => (
                        <div
                            key={idx}
                            className={`w-2 h-2 rounded-full transition-colors ${idx === currentIndex
                                    ? 'bg-accent'
                                    : answers[unresolved[idx].topic]
                                        ? 'bg-green-500'
                                        : 'bg-muted'
                                }`}
                        />
                    ))}
                </div>
            </DialogContent>
        </Dialog>
    );
}

export default ClarifyModal;
