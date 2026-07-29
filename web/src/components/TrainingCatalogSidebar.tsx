import { useEffect, useState } from "react";
import type { TrainingVariant } from "../training/catalog";
import { trainingCatalogSections } from "../training/catalog";

export function TrainingVariantThumbnail({
  variant,
}: {
  variant: TrainingVariant;
}) {
  const { bounds } = variant.map;
  const player = variant.initial.pos;

  return (
    <svg
      className="training-variant-thumbnail"
      viewBox={`${bounds.x} ${bounds.y} ${bounds.width} ${bounds.height}`}
      preserveAspectRatio="xMidYMid slice"
      role="img"
      aria-label={`${variant.title} 地图预览`}
    >
      <defs>
        <linearGradient
          id={`training-sky-${variant.id}`}
          x1="0"
          y1="0"
          x2="0"
          y2="1"
        >
          <stop offset="0" stopColor="#1a1237" />
          <stop offset="1" stopColor="#09051a" />
        </linearGradient>
        <filter
          id={`training-player-glow-${variant.id}`}
          x="-100%"
          y="-100%"
          width="300%"
          height="300%"
        >
          <feGaussianBlur stdDeviation="3" result="blur" />
          <feMerge>
            <feMergeNode in="blur" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>
      <rect {...bounds} fill={`url(#training-sky-${variant.id})`} />
      <path
        d={`M ${bounds.x} ${bounds.y + bounds.height * 0.56} L ${bounds.x + bounds.width * 0.22} ${bounds.y + bounds.height * 0.28} L ${bounds.x + bounds.width * 0.44} ${bounds.y + bounds.height * 0.62} L ${bounds.x + bounds.width * 0.67} ${bounds.y + bounds.height * 0.2} L ${bounds.x + bounds.width} ${bounds.y + bounds.height * 0.54} V ${bounds.y + bounds.height} H ${bounds.x} Z`}
        fill="#281541"
        opacity=".7"
      />
      {variant.map.solids.map((solid, index) => (
        <rect
          className="training-thumb-solid"
          key={`solid-${index}`}
          {...solid}
        />
      ))}
      {variant.map.entities.map((entity, index) => (
        <rect
          className="training-thumb-entity"
          data-kind={entity.kind}
          key={`entity-${index}`}
          {...entity.bounds}
        />
      ))}
      <circle
        className="training-thumb-player-glow"
        cx={player.x}
        cy={player.y - 7}
        r="4.5"
        filter={`url(#training-player-glow-${variant.id})`}
      />
      <path
        className="training-thumb-player"
        d={`M ${player.x} ${player.y - 12} l 5 6 -2 10 h -6 l -2 -10 z`}
      />
    </svg>
  );
}

export function TrainingCatalogSidebar({
  techniqueId,
  variantId,
  onSelectTraining,
}: {
  techniqueId: string;
  variantId: string;
  onSelectTraining(techniqueId: string, variantId: string): void;
}) {
  const [expanded, setExpanded] = useState(() => new Set([techniqueId]));

  useEffect(() => {
    setExpanded((current) =>
      current.has(techniqueId) ? current : new Set([...current, techniqueId]),
    );
  }, [techniqueId]);

  const toggleTechnique = (id: string) => {
    setExpanded((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  return (
    <aside className="training-catalog" aria-label="训练地图目录">
      <div className="training-catalog-heading">
        <small>TRAINING MAPS</small>
        <h2>训练地图</h2>
        <p>每张地图包含一个或多个教程模块。</p>
      </div>
      <nav>
        {trainingCatalogSections.map((section) => (
          <section className="training-catalog-section" key={section.id}>
            <div className="training-catalog-section-title">
              <span>{section.title}</span>
              <small>{section.badge}</small>
            </div>
            {section.techniques.map((technique) => {
              const isExpanded = expanded.has(technique.id);
              const isSelected = technique.id === techniqueId;
              return (
                <div
                  className={`training-technique ${isSelected ? "selected" : ""}`}
                  key={technique.id}
                >
                  <button
                    className="training-technique-toggle"
                    type="button"
                    aria-expanded={isExpanded}
                    onClick={() => toggleTechnique(technique.id)}
                  >
                    <span className="training-tree-point" />
                    <span>
                      <strong>{technique.title}</strong>
                      <small>{technique.summary}</small>
                    </span>
                    <i aria-hidden="true">⌄</i>
                  </button>
                  {isExpanded && (
                    <div className="training-variant-list">
                      {technique.variants.map((variant, index) => {
                        const active = isSelected && variant.id === variantId;
                        return (
                          <button
                            className={`training-variant-option ${active ? "active" : ""}`}
                            type="button"
                            aria-current={active ? "page" : undefined}
                            key={variant.id}
                            onClick={() =>
                              onSelectTraining(technique.id, variant.id)
                            }
                          >
                            <span className="training-variant-number">
                              {String(index + 1).padStart(2, "0")}
                            </span>
                            <span>
                              <strong>{variant.title}</strong>
                              <small>
                                {variant.training.modules.length} 个模块 ·{" "}
                                {variant.summary}
                              </small>
                            </span>
                          </button>
                        );
                      })}
                    </div>
                  )}
                </div>
              );
            })}
          </section>
        ))}
      </nav>
    </aside>
  );
}
