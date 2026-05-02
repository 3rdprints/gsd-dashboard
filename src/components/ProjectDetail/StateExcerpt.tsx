import { openProjectInVsCode } from "../../lib/actions";
import { Button } from "../ui/button";

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
        <Button
          type="button"
          variant="outline"
          onClick={() => void openProjectInVsCode(statePath)}
        >
          Open STATE.md
        </Button>
      </div>
    </section>
  );
}

function renderLine(line: string, index: number) {
  const key = `${index}-${line}`;
  const heading = line.match(/^(#{1,6})\s+(.+)$/);
  if (heading) {
    const headingText = heading[2];
    switch (heading[1].length) {
      case 1:
        return <h1 key={key}>{headingText}</h1>;
      case 2:
        return <h2 key={key}>{headingText}</h2>;
      case 3:
        return <h3 key={key}>{headingText}</h3>;
      case 4:
        return <h4 key={key}>{headingText}</h4>;
      case 5:
        return <h5 key={key}>{headingText}</h5>;
      default:
        return <h6 key={key}>{headingText}</h6>;
    }
  }
  if (line.trim().length === 0) {
    return <br key={key} />;
  }
  return <p key={key}>{line}</p>;
}
