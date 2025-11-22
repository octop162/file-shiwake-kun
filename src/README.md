# Frontend Structure

This directory contains the React + TypeScript frontend for ファイル仕訳け君 (File Shiwake-kun).

## Directory Structure

```
src/
├── api/              # Tauri API client wrappers
│   └── tauri.ts      # Typed wrappers for Tauri commands
├── components/       # React components
│   ├── MainWindow.tsx       # Main window with drag & drop
│   ├── SettingsPanel.tsx    # Settings panel for rule management
│   └── index.ts             # Component exports
├── context/          # React Context for state management
│   ├── AppContext.tsx       # Global application state
│   ├── ViewContext.tsx      # View/routing management
│   └── index.ts             # Context exports
├── types/            # TypeScript type definitions
│   ├── models.ts     # Data models matching Rust backend
│   ├── index.ts      # Type exports
│   └── README.md     # Type documentation
├── assets/           # Static assets
├── App.tsx           # Main application component
├── App.css           # Application styles
└── main.tsx          # Application entry point
```

## Key Components

### API Layer (`src/api/`)

Provides typed wrappers for Tauri commands:
- `processFiles()` - Process files according to rules
- `loadConfig()` - Load configuration from TOML
- `saveConfig()` - Save configuration to TOML
- `getFileInfo()` - Get file information

### Context Providers (`src/context/`)

#### AppContext
Global application state management:
- Configuration loading and saving
- Process results tracking
- Error handling
- Loading states

#### ViewContext
Simple view management (routing):
- Main view (file processing)
- Settings view (rule management)

### Components (`src/components/`)

#### MainWindow
Main window with drag & drop functionality for file processing.
- Drag & drop zone
- Progress indicators
- Results display
- Configuration info

#### SettingsPanel
Settings panel for managing organization rules.
- Rule list display
- Configuration display
- (Rule management UI to be implemented in task 13)

## Usage

### Using the App Context

```typescript
import { useApp } from './context';

function MyComponent() {
  const { config, updateConfig, isLoading, error } = useApp();
  
  // Access configuration
  console.log(config?.rules);
  
  // Update configuration
  await updateConfig(newConfig);
}
```

### Using the View Context

```typescript
import { useView, View } from './context';

function MyComponent() {
  const { currentView, goToMain, goToSettings } = useView();
  
  // Navigate to settings
  goToSettings();
  
  // Check current view
  if (currentView === View.Main) {
    // ...
  }
}
```

### Calling Tauri Commands

```typescript
import { processFiles, loadConfig } from './api/tauri';

// Process files
const results = await processFiles(['/path/to/file1', '/path/to/file2']);

// Load configuration
const config = await loadConfig();
```

## Development

### Running the Development Server

```bash
npm run dev
```

### Building for Production

```bash
npm run build
```

### Type Checking

```bash
npx tsc --noEmit
```

## Next Steps

The following tasks will build upon this structure:

- **Task 12**: Implement MainWindow component with full drag & drop functionality
- **Task 13**: Implement SettingsPanel with rule management UI
- **Task 14**: Implement ConflictDialog component
- **Task 15**: Implement processing results display
- **Task 16**: Implement preview mode UI
- **Task 17**: Add styling and UI/UX improvements

## Notes

- All components use TypeScript for type safety
- State management uses React Context API (no external state library needed)
- Tauri IPC is used for communication with the Rust backend
- The application supports both light and dark color schemes
