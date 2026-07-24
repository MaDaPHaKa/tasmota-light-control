import { ActionState } from '@model/view-state.model';
export function operationState(succeeded: number, failed: number): ActionState { if (!failed) return 'success'; if (succeeded) return 'partial_failure'; return 'failure'; }
