import { useState, useEffect } from 'react';
import { cn } from '@/lib/utils';

interface CodeEditorProps {
  value: string;
  onChange: (value: string) => void;
  language?: 'json' | 'yaml';
  placeholder?: string;
  disabled?: boolean;
  error?: string;
  className?: string;
}

export function CodeEditor({
  value,
  onChange,
  language = 'json',
  placeholder = 'Enter your plan content...',
  disabled = false,
  error,
  className,
}: CodeEditorProps) {
  const [localValue, setLocalValue] = useState(value);
  const [isDirty, setIsDirty] = useState(false);

  useEffect(() => {
    setLocalValue(value);
    setIsDirty(false);
  }, [value]);

  const handleChange = (newValue: string) => {
    setLocalValue(newValue);
    setIsDirty(true);
    onChange(newValue);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Tab') {
      e.preventDefault();
      const textarea = e.target as HTMLTextAreaElement;
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      const newValue = localValue.substring(0, start) + '  ' + localValue.substring(end);
      handleChange(newValue);
      
      // Set cursor position after the tab
      setTimeout(() => {
        textarea.selectionStart = textarea.selectionEnd = start + 2;
      }, 0);
    }
  };

  const formatContent = () => {
    if (!localValue.trim()) return;
    
    try {
      if (language === 'json') {
        const parsed = JSON.parse(localValue);
        const formatted = JSON.stringify(parsed, null, 2);
        handleChange(formatted);
      }
      // For YAML, we'd need a YAML parser/formatter
      // For now, just basic indentation cleanup
    } catch (error) {
      console.error('Failed to format content:', error);
    }
  };

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
            {language.toUpperCase()} Editor
          </span>
          {isDirty && (
            <span className="text-xs text-orange-600 dark:text-orange-400">
              Modified
            </span>
          )}
        </div>
        <button
          type="button"
          onClick={formatContent}
          disabled={disabled || !localValue.trim()}
          className="text-xs text-primary-600 hover:text-primary-700 disabled:opacity-50"
        >
          Format
        </button>
      </div>
      
      <div className="relative">
        <textarea
          value={localValue}
          onChange={(e) => handleChange(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          disabled={disabled}
          className={cn(
            'w-full h-96 px-3 py-2 text-sm font-mono border rounded-lg resize-none',
            'bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100',
            'border-gray-300 dark:border-gray-600',
            'focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500',
            'disabled:cursor-not-allowed disabled:opacity-50',
            error && 'border-red-500 focus:border-red-500 focus:ring-red-500',
            className
          )}
          style={{
            lineHeight: '1.5',
            tabSize: 2,
          }}
        />
        
        {/* Line numbers overlay */}
        <div className="absolute left-0 top-0 py-2 px-1 text-xs text-gray-400 dark:text-gray-500 font-mono pointer-events-none select-none">
          {localValue.split('\n').map((_, index) => (
            <div key={index} className="h-[1.5em] flex items-center justify-end pr-2 w-8">
              {index + 1}
            </div>
          ))}
        </div>
      </div>
      
      {error && (
        <p className="text-xs text-red-600 dark:text-red-400">
          {error}
        </p>
      )}
      
      <div className="text-xs text-gray-500 dark:text-gray-400">
        Lines: {localValue.split('\n').length} | 
        Characters: {localValue.length} | 
        Press Tab to indent
      </div>
    </div>
  );
}