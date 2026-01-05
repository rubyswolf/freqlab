import type { ChatMessage as ChatMessageType } from '../../types';

interface ChatMessageProps {
  message: ChatMessageType;
}

export function ChatMessage({ message }: ChatMessageProps) {
  const isUser = message.role === 'user';

  return (
    <div className={`flex ${isUser ? 'justify-end' : 'justify-start'}`}>
      <div
        className={`max-w-[80%] rounded-2xl px-4 py-3 ${
          isUser
            ? 'bg-accent text-white rounded-br-md'
            : 'bg-bg-tertiary text-text-primary rounded-bl-md'
        }`}
      >
        <div className="text-sm whitespace-pre-wrap break-words">{message.content}</div>
        {message.filesModified && message.filesModified.length > 0 && (
          <div className="mt-2 pt-2 border-t border-white/10">
            <div className="text-xs opacity-75 mb-1">Modified files:</div>
            {message.filesModified.map((file, i) => (
              <div key={i} className="text-xs flex items-center gap-1">
                <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                <span>{file}</span>
              </div>
            ))}
          </div>
        )}
        <div className={`text-xs mt-1 ${isUser ? 'text-white/60' : 'text-text-muted'}`}>
          {new Date(message.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
        </div>
      </div>
    </div>
  );
}
