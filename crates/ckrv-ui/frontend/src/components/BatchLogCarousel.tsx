/**
 * T045, T047: BatchLogCarousel component for displaying multiple batch log terminals
 *
 * Uses shadcn/ui Carousel to display batch logs with navigation.
 *
 * Features:
 * - Horizontal carousel navigation between batches
 * - "Batch X of Y" indicator
 * - Dot navigation indicators
 * - Auto-advance to active batch
 */

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

export interface BatchData {
    id: string;
    name: string;
    status: BatchStatus;
    logs: BatchLogEntry[];
    branch?: string;
    /** Model used for this batch (e.g., "claude-sonnet-4-20250514") */
    model?: string;
}


export interface BatchLogCarouselProps {
    /** Array of batch data to display */
    batches: BatchData[];
    /** Currently active batch ID (for auto-navigation) */
    activeBatchId?: string | null;
    /** Called when user navigates to a different batch */
    onBatchChange?: (batchId: string) => void;
}

export const BatchLogCarousel: React.FC<BatchLogCarouselProps> = ({
    batches,
    activeBatchId,
    onBatchChange,
}) => {
    const [api, setApi] = React.useState<CarouselApi>();
    const [current, setCurrent] = React.useState(0);
    const [count, setCount] = React.useState(0);

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
            return 'bg-amber-500 animate-pulse';
        case 'completed':
            return 'bg-emerald-500';
        case 'failed':
            return 'bg-red-500';
        case 'waiting':
            return 'bg-blue-500';
        default:
            return 'bg-slate-500';
    }
}

export default BatchLogCarousel;
