/**
 * @module BatchLogCarousel
 * @description
 * Carousel component for navigating between multiple batch log terminals.
 * Uses shadcn/ui Carousel with dot navigation indicators and auto-advance
 * to the currently active batch.
 *
 * @context
 * Used in ExecutionRunner to display logs from multiple batches. Allows users
 * to navigate between batches with visual status indicators showing
 * pending/running/completed/failed states.
 *
 * @dependencies
 * - BatchLogTerminal: Individual terminal display component
 * - shadcn/ui Carousel: Navigation carousel component
 *
 * @example
 * <BatchLogCarousel
 *   batches={batchData}
 *   activeBatchId={currentBatchId}
 *   onBatchChange={handleBatchChange}
 * />
 */

// === IMPORTS ===
import React, { useCallback, useEffect } from 'react';
import {
    Carousel,
    CarouselContent,
    CarouselItem,
    CarouselNext,
    CarouselPrevious,
    type CarouselApi,
} from '@/components/ui/carousel';
import { BatchLogTerminal, type BatchLogEntry, type BatchStatus } from './BatchLogTerminal';

/**
 * Data for a single batch in the carousel.
 */
export interface BatchData {
    /** Unique identifier for the batch */
    id: string;
    /** Display name for the batch */
    name: string;
    /** Current execution status */
    status: BatchStatus;
    /** Log entries for this batch */
    logs: BatchLogEntry[];
    /** Git branch name, shown when batch completes */
    branch?: string;
    /** Model identifier for display (e.g., "claude-sonnet-4-20250514") */
    model?: string;
}


/**
 * Props for the BatchLogCarousel component.
 */
export interface BatchLogCarouselProps {
    /** Array of batch data to display in the carousel */
    batches: BatchData[];
    /** ID of the currently active batch; carousel auto-navigates to it */
    activeBatchId?: string | null;
    /** Callback fired when user navigates to a different batch */
    onBatchChange?: (batchId: string) => void;
}

export const BatchLogCarousel: React.FC<BatchLogCarouselProps> = ({
    batches,
    activeBatchId,
    onBatchChange,
}) => {
    // ============================================================
    // STATE
    // ============================================================
    // === STATE ===
    /** Carousel API instance for programmatic control */
    const [api, setApi] = React.useState<CarouselApi>();
    /** Current slide index (1-based for display) */
    const [current, setCurrent] = React.useState(0);
    /** Total number of slides */
    const [count, setCount] = React.useState(0);

    // ============================================================
    // EFFECTS
    // ============================================================

    // Initialize carousel state
    useEffect(() => {
        if (!api) {
            return;
        }

        setCount(api.scrollSnapList().length);
        setCurrent(api.selectedScrollSnap() + 1);

        api.on('select', () => {
            const index = api.selectedScrollSnap();
            setCurrent(index + 1);

            // Notify parent of batch change
            if (onBatchChange && batches[index]) {
                onBatchChange(batches[index].id);
            }
        });
    }, [api, batches, onBatchChange]);

    // Auto-navigate to active batch
    useEffect(() => {
        if (!api || !activeBatchId) {
            return;
        }

        const activeIndex = batches.findIndex(b => b.id === activeBatchId);
        if (activeIndex !== -1 && activeIndex !== api.selectedScrollSnap()) {
            api.scrollTo(activeIndex);
        }
    }, [api, activeBatchId, batches]);

    // ============================================================
    // HANDLERS
    // ============================================================

    // Navigate to specific batch by index
    const goToBatch = useCallback((index: number) => {
        if (api) {
            api.scrollTo(index);
        }
    }, [api]);

    if (batches.length === 0) {
        return (
            <div className="flex items-center justify-center h-64 text-muted-foreground">
                No batches to display
            </div>
        );
    }

    return (
        <div className="flex flex-col h-full">
            {/* T047: Batch indicator */}
            <div className="flex items-center justify-between px-4 py-2 border-b border-border">
                <span className="text-sm font-medium">
                    Batch {current} of {count}
                </span>

                {/* T047: Dot navigation indicators */}
                <div className="flex items-center gap-1.5">
                    {batches.map((batch, index) => {
                        const isActive = index === current - 1;
                        const statusColor = getStatusDotColor(batch.status);

                        return (
                            <button
                                key={batch.id}
                                onClick={() => goToBatch(index)}
                                className={`
                                    w-2 h-2 rounded-full transition-all
                                    ${isActive ? 'w-4' : ''}
                                    ${statusColor}
                                `}
                                title={`${batch.name} (${batch.status})`}
                            />
                        );
                    })}
                </div>
            </div>

            {/* Carousel */}
            <div className="flex-1 min-h-0 px-4 py-2">
                <Carousel
                    setApi={setApi}
                    className="w-full h-full"
                    opts={{
                        align: 'start',
                        loop: false,
                    }}
                >
                    <CarouselContent className="-ml-2 h-full">
                        {batches.map((batch, index) => (
                            <CarouselItem key={batch.id} className="pl-2 h-full">
                                <BatchLogTerminal
                                    batchId={batch.id}
                                    batchName={batch.name}
                                    batchIndex={index}
                                    status={batch.status}
                                    logs={batch.logs}
                                    branch={batch.branch}
                                    model={batch.model}
                                    autoScroll={batch.status === 'running'}
                                />
                            </CarouselItem>
                        ))}
                    </CarouselContent>

                    {/* Navigation buttons with z-index to stay above content */}
                    {batches.length > 1 && (
                        <>
                            <CarouselPrevious className="left-0 z-10" />
                            <CarouselNext className="right-0 z-10" />
                        </>
                    )}
                </Carousel>
            </div>
        </div>
    );
};

function getStatusDotColor(status: BatchStatus): string {
    switch (status) {
        case 'running':
            return 'bg-warning animate-pulse';
        case 'completed':
            return 'bg-success';
        case 'failed':
            return 'bg-error';
        case 'waiting':
            return 'bg-primary';
        default:
            return 'bg-muted-foreground';
    }
}

export default BatchLogCarousel;
