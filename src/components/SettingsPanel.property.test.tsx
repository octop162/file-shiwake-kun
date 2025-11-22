/**
 * SettingsPanelコンポーネントのプロパティベーステスト
 * Property-based tests for SettingsPanel component
 * 
 * Feature: file-shiwake-kun, Property 12: ルール変更の反映
 * Validates: Requirements 6.3, 6.4, 6.5
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import * as fc from 'fast-check';
import SettingsPanel from './SettingsPanel';
import { AppProvider } from '../context';
import { Rule, OperationType } from '../types';

// Mock the Tauri API
let mockConfig = {
  rules: [] as Rule[],
  default_destination: 'C:/Test',
  preview_mode: false,
  log_path: 'test.log',
};

const mockSaveConfig = vi.fn().mockImplementation((config) => {
  mockConfig = config;
  return Promise.resolve();
});

const mockLoadConfig = vi.fn().mockImplementation(() => {
  return Promise.resolve(mockConfig);
});

vi.mock('../api/tauri', () => ({
  loadConfig: () => mockLoadConfig(),
  saveConfig: (config: any) => mockSaveConfig(config),
  processFiles: vi.fn().mockResolvedValue([]),
}));

/**
 * Arbitrary for generating valid Rule objects
 */
const ruleArbitrary: fc.Arbitrary<Rule> = fc.record({
  id: fc.string({ minLength: 1, maxLength: 20 }).map(s => `rule-${s}`),
  name: fc.string({ minLength: 1, maxLength: 50 }).filter(s => s.trim().length > 0), // Ensure non-whitespace names
  priority: fc.integer({ min: 1, max: 100 }),
  conditions: fc.array(
    fc.record({
      field: fc.constantFrom('extension', 'size', 'capture_date', 'camera_model'),
      operator: fc.constantFrom('==', '!=', 'in', 'exists'),
      value: fc.oneof(
        fc.string(),
        fc.integer(),
        fc.array(fc.string())
      ),
    }),
    { minLength: 0, maxLength: 5 }
  ),
  destination_pattern: fc.string({ minLength: 1, maxLength: 100 }).filter(s => s.trim().length > 0), // Ensure non-whitespace patterns
  operation: fc.constantFrom(OperationType.Move, OperationType.Copy),
});

/**
 * テスト用のラッパーコンポーネント
 */
const renderWithProvider = (ui: React.ReactElement) => {
  return render(<AppProvider>{ui}</AppProvider>);
};

describe('SettingsPanel Property-Based Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockConfig = {
      rules: [],
      default_destination: 'C:/Test',
      preview_mode: false,
      log_path: 'test.log',
    };
  });

  /**
   * Feature: file-shiwake-kun, Property 12: ルール変更の反映
   * Validates: Requirements 6.3, 6.4, 6.5
   * 
   * Property: For any rule addition, the config should immediately reflect the new rule
   */
  it('should immediately reflect rule additions in config', async () => {
    await fc.assert(
      fc.asyncProperty(ruleArbitrary, async (generatedRule) => {
        // Reset mock config
        mockConfig = {
          rules: [],
          default_destination: 'C:/Test',
          preview_mode: false,
          log_path: 'test.log',
        };
        vi.clearAllMocks();

        const { unmount } = renderWithProvider(<SettingsPanel />);

        // Wait for component to load
        await waitFor(() => {
          expect(screen.getByText('＋ ルールを追加')).toBeInTheDocument();
        });

        // Click add rule button
        const addButton = screen.getByText('＋ ルールを追加');
        fireEvent.click(addButton);

        // Fill in the form with generated rule data
        const nameInput = screen.getByPlaceholderText('例: 写真を年月別に整理');
        fireEvent.change(nameInput, { target: { value: generatedRule.name } });

        const patternInput = screen.getByPlaceholderText('例: D:/Photos/{year}/{month}');
        fireEvent.change(patternInput, { target: { value: generatedRule.destination_pattern } });

        // Submit the form
        const submitButton = screen.getByText('追加');
        fireEvent.click(submitButton);

        // Wait for save to be called
        await waitFor(() => {
          expect(mockSaveConfig).toHaveBeenCalled();
        });

        // Verify that the saved config contains the new rule
        const savedConfig = mockSaveConfig.mock.calls[0][0];
        expect(savedConfig.rules).toHaveLength(1);
        expect(savedConfig.rules[0].name).toBe(generatedRule.name);
        expect(savedConfig.rules[0].destination_pattern).toBe(generatedRule.destination_pattern);

        unmount();
      }),
      { numRuns: 10 } // Reduced for performance
    );
  }, 60000); // 60 second timeout

  /**
   * Feature: file-shiwake-kun, Property 12: ルール変更の反映
   * Validates: Requirements 6.3, 6.4, 6.5
   * 
   * Property: For any rule deletion, the config should immediately reflect the removal
   */
  it('should immediately reflect rule deletions in config', async () => {
    await fc.assert(
      fc.asyncProperty(
        fc.array(ruleArbitrary, { minLength: 1, maxLength: 3 }),
        fc.integer({ min: 0, max: 2 }),
        async (initialRules, deleteIndexRaw) => {
          const deleteIndex = deleteIndexRaw % initialRules.length;

          // Set up initial config with rules
          mockConfig = {
            rules: initialRules,
            default_destination: 'C:/Test',
            preview_mode: false,
            log_path: 'test.log',
          };
          vi.clearAllMocks();

          // Mock window.confirm to always return true
          const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);

          const { unmount, container } = renderWithProvider(<SettingsPanel />);

          try {
            // Wait for rules to be displayed using container query
            await waitFor(() => {
              const deleteButtons = container.querySelectorAll('[title="削除"]');
              expect(deleteButtons.length).toBe(initialRules.length);
            });

            // Get all delete buttons and click the one at deleteIndex
            const deleteButtons = container.querySelectorAll('[title="削除"]');
            fireEvent.click(deleteButtons[deleteIndex]);

            // Wait for save to be called
            await waitFor(() => {
              expect(mockSaveConfig).toHaveBeenCalled();
            });

            // Verify that the saved config has one less rule
            const savedConfig = mockSaveConfig.mock.calls[0][0];
            expect(savedConfig.rules).toHaveLength(initialRules.length - 1);

            // Verify that the deleted rule is not in the saved config
            const deletedRuleId = initialRules[deleteIndex].id;
            expect(savedConfig.rules.find((r: Rule) => r.id === deletedRuleId)).toBeUndefined();
          } finally {
            confirmSpy.mockRestore();
            unmount();
          }
        }
      ),
      { numRuns: 10 } // Reduced for performance
    );
  }, 60000); // 60 second timeout

  /**
   * Feature: file-shiwake-kun, Property 12: ルール変更の反映
   * Validates: Requirements 6.3, 6.4, 6.5
   * 
   * Property: For any rule edit, the config should immediately reflect the changes
   */
  it('should immediately reflect rule edits in config', async () => {
    await fc.assert(
      fc.asyncProperty(
        fc.array(ruleArbitrary, { minLength: 1, maxLength: 3 }),
        fc.integer({ min: 0, max: 2 }),
        fc.string({ minLength: 1, maxLength: 50 }).filter(s => s.trim().length > 0),
        async (initialRules, editIndexRaw, newName) => {
          const editIndex = editIndexRaw % initialRules.length;

          // Set up initial config with rules
          mockConfig = {
            rules: initialRules,
            default_destination: 'C:/Test',
            preview_mode: false,
            log_path: 'test.log',
          };
          vi.clearAllMocks();

          const { unmount, container } = renderWithProvider(<SettingsPanel />);

          try {
            // Wait for rules to be displayed using a more flexible matcher
            await waitFor(() => {
              const editButtons = container.querySelectorAll('[title="編集"]');
              expect(editButtons.length).toBeGreaterThan(0);
            });

            // Get all edit buttons and click the one at editIndex
            const editButtons = container.querySelectorAll('[title="編集"]');
            fireEvent.click(editButtons[editIndex]);

            // Wait for edit form to appear
            await waitFor(() => {
              expect(screen.getByText('ルールを編集')).toBeInTheDocument();
            });

            // Change the name using placeholder
            const nameInput = screen.getByPlaceholderText('例: 写真を年月別に整理');
            fireEvent.change(nameInput, { target: { value: newName } });

            // Save the edit
            const saveButton = screen.getByText('保存');
            fireEvent.click(saveButton);

            // Wait for save to be called
            await waitFor(() => {
              expect(mockSaveConfig).toHaveBeenCalled();
            });

            // Verify that the saved config has the updated rule
            const savedConfig = mockSaveConfig.mock.calls[0][0];
            expect(savedConfig.rules).toHaveLength(initialRules.length);
            expect(savedConfig.rules[editIndex].name).toBe(newName);
            expect(savedConfig.rules[editIndex].id).toBe(initialRules[editIndex].id);
          } finally {
            unmount();
          }
        }
      ),
      { numRuns: 10 } // Reduced for performance
    );
  }, 60000); // 60 second timeout

  /**
   * Feature: file-shiwake-kun, Property 12: ルール変更の反映
   * Validates: Requirements 6.3, 6.4, 6.5
   * 
   * Property: For any rule reordering, the config should immediately reflect the new order
   */
  it('should immediately reflect rule reordering in config', async () => {
    await fc.assert(
      fc.asyncProperty(
        fc.array(ruleArbitrary, { minLength: 2, maxLength: 3 }),
        fc.integer({ min: 0, max: 2 }),
        fc.integer({ min: 0, max: 2 }),
        async (initialRules, fromIndexRaw, toIndexRaw) => {
          if (initialRules.length < 2) return; // Skip if not enough rules

          const fromIndex = fromIndexRaw % initialRules.length;
          const toIndex = toIndexRaw % initialRules.length;

          if (fromIndex === toIndex) return; // Skip if same index

          // Set up initial config with rules
          mockConfig = {
            rules: initialRules,
            default_destination: 'C:/Test',
            preview_mode: false,
            log_path: 'test.log',
          };
          vi.clearAllMocks();

          const { unmount, container } = renderWithProvider(<SettingsPanel />);

          try {
            // Wait for rules to be displayed using container query
            await waitFor(() => {
              const ruleItems = container.querySelectorAll('.rules-list li');
              expect(ruleItems.length).toBe(initialRules.length);
            });

            // Get all rule list items
            const ruleItems = container.querySelectorAll('.rules-list li');

            // Simulate drag and drop
            const draggedItem = ruleItems[fromIndex];
            const dropTarget = ruleItems[toIndex];

            // Trigger drag start
            fireEvent.dragStart(draggedItem);

            // Trigger drop
            const dropEvent = new Event('drop', { bubbles: true }) as any;
            dropEvent.dataTransfer = { files: [] };
            fireEvent(dropTarget, dropEvent);

            // Wait for save to be called
            await waitFor(() => {
              expect(mockSaveConfig).toHaveBeenCalled();
            }, { timeout: 3000 });

            // Verify that the saved config has the reordered rules
            const savedConfig = mockSaveConfig.mock.calls[0][0];
            expect(savedConfig.rules).toHaveLength(initialRules.length);

            // Calculate expected order
            const expectedRules = [...initialRules];
            const [movedRule] = expectedRules.splice(fromIndex, 1);
            expectedRules.splice(toIndex, 0, movedRule);

            // Verify the order matches
            expect(savedConfig.rules.map((r: Rule) => r.id)).toEqual(
              expectedRules.map(r => r.id)
            );
          } finally {
            unmount();
          }
        }
      ),
      { numRuns: 5 } // Reduced runs for drag-drop tests as they're slower
    );
  }, 60000); // 60 second timeout
});
