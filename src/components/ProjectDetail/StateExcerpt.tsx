import { openProjectInVsCode } from "../../lib/actions";

type StateExcerptProps = {
  statePath: string;
  excerpt: string | null;
};

export function StateExcerpt({ statePath, excerpt }: StateExcerptProps) {
  const lines = (excerpt ?? "Current position is not available.").split(/\r?\n/);

  return (
    <section className="state-excerpt" aria-label="Current Position">
      <p className="label-text">Current Position</p>
      <div>
        {lines.map((line, index) => renderLine(line, index))}
      </div>
      <div className="state-excerpt-overflow">
        <button
          type="button"
          className="secondary-button"
          onClick={() => void openProjectInVsCode(statePath)}
        >
          Open STATE.md
        </button>
      </div>
    </section>
  );
}

function renderLine(line: string, index: number) {
  const key = `${index}-${line}`;
  const heading = line.match(/^#{1,6}\s+(.+)$/);
  if (heading) {
    return <h2 key={key}>{heading[1]}</h2>;
  }
  if (line.trim().length === 0) {
    return <br key={key} />;
  }
  return <p key={key}>{line}</p>;
}
