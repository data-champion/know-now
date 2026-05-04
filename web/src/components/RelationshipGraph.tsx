import { useCallback, useEffect, useRef, useState } from "react";
import type { GraphEdge, GraphNode } from "../api/client";

interface RelationshipGraphProps {
  nodes: GraphNode[];
  edges: GraphEdge[];
  onNodeSelect: (nodeId: string) => void;
  selectedNodeId: string | null;
}

interface LayoutNode {
  id: string;
  name: string;
  domain: string | null;
  x: number;
  y: number;
  vx: number;
  vy: number;
}

export function RelationshipGraph({
  nodes,
  edges,
  onNodeSelect,
  selectedNodeId,
}: RelationshipGraphProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const [layoutNodes, setLayoutNodes] = useState<LayoutNode[]>([]);
  const [viewBox, setViewBox] = useState({ x: -300, y: -300, w: 600, h: 600 });
  const [hoveredNode, setHoveredNode] = useState<string | null>(null);
  const [focusedIndex, setFocusedIndex] = useState(0);
  const dragRef = useRef<{ startX: number; startY: number; vbX: number; vbY: number } | null>(null);

  useEffect(() => {
    if (nodes.length === 0) return;
    const angle = (2 * Math.PI) / nodes.length;
    const radius = Math.max(100, nodes.length * 20);
    const initial: LayoutNode[] = nodes.map((n, i) => ({
      ...n,
      x: Math.cos(angle * i) * radius,
      y: Math.sin(angle * i) * radius,
      vx: 0,
      vy: 0,
    }));
    runSimulation(initial, edges, setLayoutNodes);
  }, [nodes, edges]);

  const handleMouseDown = useCallback((e: React.MouseEvent<SVGSVGElement>) => {
    if ((e.target as Element).closest(".kn-graph__node")) return;
    dragRef.current = { startX: e.clientX, startY: e.clientY, vbX: viewBox.x, vbY: viewBox.y };
  }, [viewBox]);

  const handleMouseMove = useCallback((e: React.MouseEvent<SVGSVGElement>) => {
    const drag = dragRef.current;
    if (!drag || !svgRef.current) return;
    const rect = svgRef.current.getBoundingClientRect();
    const scaleX = viewBox.w / rect.width;
    const scaleY = viewBox.h / rect.height;
    const dx = (e.clientX - drag.startX) * scaleX;
    const dy = (e.clientY - drag.startY) * scaleY;
    setViewBox((vb) => ({ ...vb, x: drag.vbX - dx, y: drag.vbY - dy }));
  }, [viewBox.w, viewBox.h]);

  const handleMouseUp = useCallback(() => {
    dragRef.current = null;
  }, []);

  const handleWheel = useCallback((e: React.WheelEvent<SVGSVGElement>) => {
    e.preventDefault();
    const factor = e.deltaY > 0 ? 1.1 : 0.9;
    setViewBox((vb) => {
      const cx = vb.x + vb.w / 2;
      const cy = vb.y + vb.h / 2;
      const nw = vb.w * factor;
      const nh = vb.h * factor;
      return { x: cx - nw / 2, y: cy - nh / 2, w: nw, h: nh };
    });
  }, []);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (layoutNodes.length === 0) return;
      if (e.key === "ArrowRight" || e.key === "ArrowDown") {
        e.preventDefault();
        setFocusedIndex((i) => (i + 1) % layoutNodes.length);
      } else if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
        e.preventDefault();
        setFocusedIndex((i) => (i - 1 + layoutNodes.length) % layoutNodes.length);
      } else if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        const node = layoutNodes[focusedIndex];
        if (node) onNodeSelect(node.id);
      }
    },
    [layoutNodes, focusedIndex, onNodeSelect],
  );

  if (nodes.length === 0) {
    return <p className="kn-empty">No graph data available.</p>;
  }

  const nodeMap = new Map(layoutNodes.map((n) => [n.id, n]));

  return (
    <div className="kn-graph" role="img" aria-label="Entity relationship graph">
      <svg
        ref={svgRef}
        className="kn-graph__svg"
        viewBox={`${String(viewBox.x)} ${String(viewBox.y)} ${String(viewBox.w)} ${String(viewBox.h)}`}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
        onWheel={handleWheel}
        onKeyDown={handleKeyDown}
        tabIndex={0}
        aria-label="Graph canvas — use arrow keys to navigate nodes, Enter to select"
      >
        <g className="kn-graph__edges">
          {edges.map((edge) => {
            const from = nodeMap.get(edge.from);
            const to = nodeMap.get(edge.to);
            if (!from || !to) return null;
            return (
              <line
                key={edge.id}
                x1={from.x}
                y1={from.y}
                x2={to.x}
                y2={to.y}
                className="kn-graph__edge"
                markerEnd="url(#arrowhead)"
              />
            );
          })}
        </g>
        <defs>
          <marker
            id="arrowhead"
            markerWidth="8"
            markerHeight="6"
            refX="20"
            refY="3"
            orient="auto"
          >
            <polygon points="0 0, 8 3, 0 6" className="kn-graph__arrow" />
          </marker>
        </defs>
        <g className="kn-graph__nodes">
          {layoutNodes.map((node, i) => {
            const isSelected = selectedNodeId === node.id;
            const isHovered = hoveredNode === node.id;
            const isFocused = focusedIndex === i;
            return (
              <g
                key={node.id}
                className={`kn-graph__node${isSelected ? " kn-graph__node--selected" : ""}${isFocused ? " kn-graph__node--focused" : ""}`}
                transform={`translate(${String(node.x)},${String(node.y)})`}
                onClick={() => { onNodeSelect(node.id); }}
                onMouseEnter={() => { setHoveredNode(node.id); }}
                onMouseLeave={() => { setHoveredNode(null); }}
                role="button"
                aria-label={`${node.name}${node.domain ? ` (${node.domain})` : ""}`}
              >
                <circle r="14" className="kn-graph__circle" />
                <text
                  dy="4"
                  textAnchor="middle"
                  className="kn-graph__label"
                >
                  {node.name.length > 12 ? `${node.name.slice(0, 11)}…` : node.name}
                </text>
                {(isHovered || isFocused) && (
                  <title>{`${node.name}${node.domain ? ` — domain: ${node.domain}` : ""}`}</title>
                )}
              </g>
            );
          })}
        </g>
      </svg>
    </div>
  );
}

function runSimulation(
  initial: LayoutNode[],
  edges: GraphEdge[],
  onDone: (nodes: LayoutNode[]) => void,
) {
  const nodes = initial.map((n) => ({ ...n }));
  const iterations = 120;
  const repulsion = 5000;
  const attraction = 0.005;
  const damping = 0.9;

  for (let iter = 0; iter < iterations; iter++) {
    for (let i = 0; i < nodes.length; i++) {
      const nodeI = nodes[i];
      if (!nodeI) continue;
      for (let j = i + 1; j < nodes.length; j++) {
        const nodeJ = nodes[j];
        if (!nodeJ) continue;
        const dx = nodeI.x - nodeJ.x;
        const dy = nodeI.y - nodeJ.y;
        const dist = Math.max(Math.sqrt(dx * dx + dy * dy), 1);
        const force = repulsion / (dist * dist);
        const fx = (dx / dist) * force;
        const fy = (dy / dist) * force;
        nodeI.vx += fx;
        nodeI.vy += fy;
        nodeJ.vx -= fx;
        nodeJ.vy -= fy;
      }
    }

    const nodeIndex = new Map(nodes.map((n, idx) => [n.id, idx]));
    for (const edge of edges) {
      const si = nodeIndex.get(edge.from);
      const ti = nodeIndex.get(edge.to);
      if (si === undefined || ti === undefined) continue;
      const source = nodes[si];
      const target = nodes[ti];
      if (!source || !target) continue;
      const dx = target.x - source.x;
      const dy = target.y - source.y;
      const fx = dx * attraction;
      const fy = dy * attraction;
      source.vx += fx;
      source.vy += fy;
      target.vx -= fx;
      target.vy -= fy;
    }

    for (const node of nodes) {
      node.vx *= damping;
      node.vy *= damping;
      node.x += node.vx;
      node.y += node.vy;
    }
  }

  onDone(nodes);
}
