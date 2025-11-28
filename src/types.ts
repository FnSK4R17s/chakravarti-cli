export interface Agent {
    name: string;
    description?: string;
    type: 'cli' | 'ide';
    check: () => Promise<boolean>;
}

export interface CheckResult {
    agent: Agent;
    available: boolean;
}
