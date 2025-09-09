"use client";

import { cn } from "@/lib/utils";
import { FC, HTMLAttributes, ReactNode } from "react";

interface BorderBeamProps extends HTMLAttributes<HTMLDivElement> {
	children: ReactNode;
	size?: number;
	duration?: number;
	borderWidth?: number;
	anchor?: number;
	colorFrom?: string;
	colorTo?: string;
	delay?: number;
}

export const BorderBeam: FC<BorderBeamProps> = ({
	children,
	className,
	size = 200,
	duration = 15,
	anchor = 90,
	borderWidth = 1.5,
	colorFrom = "#ffaa40",
	colorTo = "#9c40ff",
	delay = 0,
	...props
}) => {
	return (
		<div
			style={
				{
					"--size": size,
					"--duration": duration,
					"--anchor": anchor,
					"--border-width": borderWidth,
					"--color-from": colorFrom,
					"--color-to": colorTo,
					"--delay": `-${delay}s`,
				} as React.CSSProperties
			}
			className={cn(
				"relative flex min-h-[60px] w-fit min-w-[270px] items-center justify-center rounded-xl border border-transparent bg-white bg-dot-black/[0.2] p-10 text-center dark:border-white/[0.2] dark:bg-black dark:bg-dot-white/[0.2]",
				"before:absolute before:inset-0 before:rounded-xl before:border-[--border-width] before:border-transparent",
				"before:bg-[conic-gradient(from_var(--anchor)_at_50%_50%,transparent_0deg,var(--color-from)_90deg,var(--color-to)_180deg,transparent_270deg)]",
				"before:bg-size-[length:var(--size)%_var(--size)%] before:bg-repeat-[no-repeat] before:bg-position-[0%_0%]",
				"before:animate-border-beam before:[animation-delay:var(--delay)]",
				className,
			)}
			{...props}
		>
			{children}
		</div>
	);
};
