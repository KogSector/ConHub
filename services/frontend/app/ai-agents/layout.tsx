import React from 'react';

export default function AIAgentsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="ai-agents-container">
      <div className="ai-agents-content">
        {children}
      </div>
    </div>
  );
}