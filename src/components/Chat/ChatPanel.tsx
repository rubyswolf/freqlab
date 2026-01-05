import { useState, useRef, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { ChatMessage } from './ChatMessage';
import { ChatInput } from './ChatInput';
import { useOutputStore } from '../../stores/outputStore';
import { useChatStore } from '../../stores/chatStore';
import type { ChatMessage as ChatMessageType } from '../../types';
import type { ProjectMeta } from '../../types';

interface ClaudeStreamEvent {
  type: 'start' | 'text' | 'error' | 'done';
  content?: string;
  message?: string;
}

interface ChatPanelProps {
  project: ProjectMeta;
}

export function ChatPanel({ project }: ChatPanelProps) {
  const [messages, setMessages] = useState<ChatMessageType[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [streamingContent, setStreamingContent] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const { addLine, setActive, clear } = useOutputStore();
  const { pendingMessage, clearPendingMessage } = useChatStore();

  // Auto-scroll to bottom when messages change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, streamingContent]);

  const handleSend = useCallback(async (content: string) => {
    // Add user message
    const userMessage: ChatMessageType = {
      id: crypto.randomUUID(),
      role: 'user',
      content,
      timestamp: new Date().toISOString(),
    };
    setMessages((prev) => [...prev, userMessage]);
    setIsLoading(true);
    setStreamingContent('');

    // Clear and activate output panel
    clear();
    setActive(true);
    addLine(`> Sending to Claude: "${content.substring(0, 50)}${content.length > 50 ? '...' : ''}"`);
    addLine('');

    // Listen for streaming events
    const unlisten = await listen<ClaudeStreamEvent>('claude-stream', (event) => {
      const data = event.payload;
      if (data.type === 'text' && data.content) {
        setStreamingContent((prev) => prev + data.content + '\n');
        addLine(data.content);
      } else if (data.type === 'error' && data.message) {
        setStreamingContent((prev) => prev + `\nError: ${data.message}`);
        addLine(`[ERROR] ${data.message}`);
      } else if (data.type === 'start') {
        addLine('[Claude started working...]');
      }
    });

    try {
      const response = await invoke<{ content: string }>('send_to_claude', {
        projectPath: project.path,
        projectName: project.name,
        description: project.description,
        message: content,
      });

      // Add assistant message
      const assistantMessage: ChatMessageType = {
        id: crypto.randomUUID(),
        role: 'assistant',
        content: response.content.trim() || streamingContent.trim(),
        timestamp: new Date().toISOString(),
      };
      setMessages((prev) => [...prev, assistantMessage]);
    } catch (err) {
      // Add error message
      const errorMessage: ChatMessageType = {
        id: crypto.randomUUID(),
        role: 'assistant',
        content: `Sorry, something went wrong: ${err}`,
        timestamp: new Date().toISOString(),
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      unlisten();
      setIsLoading(false);
      setStreamingContent('');
      setActive(false);
      addLine('');
      addLine('[Done]');
    }
  }, [project, addLine, setActive, clear]);

  // Watch for pending messages (e.g., from "Fix with Claude" button)
  useEffect(() => {
    if (pendingMessage && !isLoading) {
      handleSend(pendingMessage);
      clearPendingMessage();
    }
  }, [pendingMessage, isLoading, handleSend, clearPendingMessage]);

  return (
    <div className="h-full flex flex-col bg-bg-secondary rounded-xl border border-border overflow-hidden">
      {/* Messages area */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 && !isLoading ? (
          <div className="h-full flex items-center justify-center">
            <div className="text-center max-w-md">
              <div className="w-16 h-16 mx-auto rounded-xl bg-bg-tertiary flex items-center justify-center mb-4">
                <svg className="w-8 h-8 text-accent" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.5}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M8.625 12a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0H8.25m4.125 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0H12m4.125 0a.375.375 0 11-.75 0 .375.375 0 01.75 0zm0 0h-.375M21 12c0 4.556-4.03 8.25-9 8.25a9.764 9.764 0 01-2.555-.337A5.972 5.972 0 015.41 20.97a5.969 5.969 0 01-.474-.065 4.48 4.48 0 00.978-2.025c.09-.457-.133-.901-.467-1.226C3.93 16.178 3 14.189 3 12c0-4.556 4.03-8.25 9-8.25s9 3.694 9 8.25z" />
                </svg>
              </div>
              <h3 className="text-lg font-medium text-text-primary mb-2">Start building {project.name}</h3>
              <p className="text-sm text-text-muted">
                Describe what features you want to add. Claude will modify the plugin code for you.
              </p>
              <div className="mt-4 text-xs text-text-muted">
                <p className="mb-1">Try something like:</p>
                <p className="text-accent">"Add a low-pass filter with resonance"</p>
                <p className="text-accent">"Make it a stereo delay with feedback"</p>
              </div>
            </div>
          </div>
        ) : (
          <>
            {messages.map((message) => (
              <ChatMessage key={message.id} message={message} />
            ))}
            {isLoading && (
              <div className="flex justify-start">
                <div className="rounded-2xl rounded-bl-md px-4 py-3 bg-bg-tertiary">
                  <div className="flex items-center gap-2 text-text-muted">
                    <div className="w-2 h-2 rounded-full bg-accent animate-pulse" />
                    <span className="text-sm">Claude is working...</span>
                  </div>
                  <p className="text-xs text-text-muted mt-2">
                    View progress in the output panel below
                  </p>
                </div>
              </div>
            )}
            <div ref={messagesEndRef} />
          </>
        )}
      </div>

      {/* Input area */}
      <ChatInput
        onSend={handleSend}
        disabled={isLoading}
        placeholder={messages.length === 0 ? 'Describe what you want to build...' : 'Ask for changes...'}
      />
    </div>
  );
}
