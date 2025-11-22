# Type Definitions

This directory contains TypeScript type definitions that match the Rust data models used in the Tauri backend.

## Files

- `models.ts` - Core data model types
- `index.ts` - Re-exports all types for easy importing

## Usage

```typescript
import { Config, Rule, FileMetadata, ProcessResult } from './types';
```

## Data Models

### Rule
Defines a file organization rule with conditions and destination pattern.

### Config
Application configuration including rules, default destination, and settings.

### FileMetadata
File metadata including filesystem attributes (size, dates) and EXIF data (for images).

### ProcessResult
Result of a file processing operation, including success status and error messages.

### FileInfo
Basic file information (name, size, modification time).

### ConflictResolution
Enum for handling file conflicts when destination file already exists.

## Serialization

All types are designed to be serialized/deserialized via JSON when communicating between the React frontend and Rust backend through Tauri IPC.

SystemTime values from Rust are converted to Unix timestamps (milliseconds) in TypeScript.
