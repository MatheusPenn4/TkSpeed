import type { ButtonHTMLAttributes } from "react";
import { AxIcon, type AxIconName } from "./AxIcon";

type Props = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: "default" | "primary" | "ghost";
  size?: "md" | "sm";
  icon?: AxIconName;
};

export function AxButton({ variant = "default", size = "md", icon, children, className = "", ...rest }: Props) {
  const cls = [
    "ax-btn",
    variant !== "default" ? `ax-btn-${variant}` : "",
    size === "sm" ? "ax-btn-sm" : "",
    className,
  ]
    .filter(Boolean)
    .join(" ");
  return (
    <button className={cls} {...rest}>
      {icon && <AxIcon name={icon} size={size === "sm" ? 14 : 16} />}
      {children}
    </button>
  );
}
