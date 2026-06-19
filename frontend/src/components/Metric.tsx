interface Props {
  label: string;
  value: string | number;
  unit?: string;
  big?: boolean;
  warn?: boolean;
}

/** A single labeled reading. `big` for the headline number on a card. */
export function Metric({ label, value, unit, big, warn }: Props) {
  return (
    <div className={`metric${big ? " metric--big" : ""}${warn ? " metric--warn" : ""}`}>
      <div className="metric__value">
        {value}
        {unit && <span className="metric__unit">{unit}</span>}
      </div>
      <div className="metric__label">{label}</div>
    </div>
  );
}
