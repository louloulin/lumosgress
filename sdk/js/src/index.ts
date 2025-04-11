/**
 * Proksi AI Gateway SDK
 * Main entry point for the JavaScript/TypeScript SDK
 */

// Export the client class
export { ProksiClient } from './client';

// Export all types
export * from './types';

// Default export for convenience
import { ProksiClient } from './client';
export default ProksiClient; 