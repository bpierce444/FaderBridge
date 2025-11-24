/**
 * Tests for Layout component
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Layout } from './Layout';

describe('Layout', () => {
  it('renders all panels', () => {
    render(
      <Layout
        leftPanel={<div>Left Panel</div>}
        centerPanel={<div>Center Panel</div>}
        rightPanel={<div>Right Panel</div>}
      />
    );

    expect(screen.getByText('Left Panel')).toBeInTheDocument();
    expect(screen.getByText('Center Panel')).toBeInTheDocument();
    expect(screen.getByText('Right Panel')).toBeInTheDocument();
  });

  it('renders top bar when provided', () => {
    render(
      <Layout
        topBar={<div>Top Bar</div>}
        leftPanel={<div>Left</div>}
        centerPanel={<div>Center</div>}
        rightPanel={<div>Right</div>}
      />
    );

    expect(screen.getByText('Top Bar')).toBeInTheDocument();
  });

  it('renders status bar when provided', () => {
    render(
      <Layout
        leftPanel={<div>Left</div>}
        centerPanel={<div>Center</div>}
        rightPanel={<div>Right</div>}
        statusBar={<div>Status Bar</div>}
      />
    );

    expect(screen.getByText('Status Bar')).toBeInTheDocument();
  });

  it('applies correct layout structure', () => {
    const { container } = render(
      <Layout
        leftPanel={<div>Left</div>}
        centerPanel={<div>Center</div>}
        rightPanel={<div>Right</div>}
      />
    );

    // Check for main container with full height
    const mainContainer = container.querySelector('.h-screen');
    expect(mainContainer).toBeInTheDocument();

    // Check for flex layout
    const contentArea = container.querySelector('.flex-1.flex');
    expect(contentArea).toBeInTheDocument();
  });

  it('renders without top and status bars', () => {
    render(
      <Layout
        leftPanel={<div>Left</div>}
        centerPanel={<div>Center</div>}
        rightPanel={<div>Right</div>}
      />
    );

    // Should still render the three main panels
    expect(screen.getByText('Left')).toBeInTheDocument();
    expect(screen.getByText('Center')).toBeInTheDocument();
    expect(screen.getByText('Right')).toBeInTheDocument();
  });
});
