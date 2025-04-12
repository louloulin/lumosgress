/**
 * Proksi AI Gateway SDK
 * Main entry point for the JavaScript/TypeScript SDK
 */

// Export core client
export { ProksiClient } from './client';

// Export types
export * from './types';

// Import CLI (not exported directly since it's used as a binary)
import './cli';

// Default export for convenience
import { ProksiClient } from './client';
export default ProksiClient; 