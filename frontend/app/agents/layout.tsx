import React from 'react';

export default function AgentsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="agents-container">
      <div className="agents-content">
        {children}
      </div>
    </div>
  );
}