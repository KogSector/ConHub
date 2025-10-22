"use client";

import { cn } from "@/lib/utils";
import { HTMLAttributes, ReactNode } from "react";

interface ShimmerButtonProps extends HTMLAttributes<HTMLButtonElement> {
	children: ReactNode;
	shimmerColor?: string;
	shimmerSize?: string;
	borderRadius?: string;
	shimmerDuration?: string;
	background?: string;
	className?: string;
}

export const ShimmerButton = ({
	children,
	shimmerColor = "#ffffff",
	shimmerSize = "0.1em",
	shimmerDuration = "1.5s",
	borderRadius = "100px",
	background = "rgba(0, 0, 0, 1)",
	className,
	...props
}: ShimmerButtonProps) => {
	return (
		<button
			style={
				{
					"--shimmer-color": shimmerColor,
					"--shimmer-size": shimmerSize,
					"--shimmer-duration": shimmerDuration,
					"--border-radius": borderRadius,
					"--background": background,
				} as React.CSSProperties
			}
			className={cn(
				"group relative z-0 flex cursor-pointer items-center justify-center overflow-hidden whitespace-nowrap border border-white/10 px-6 py-3 text-white [border-radius:var(--border-radius)] [background:var(--background)]",
				"transform-gpu transition-transform duration-300 ease-in-out active:translate-y-px",
				"before:absolute before:left-0 before:top-0 before:-z-10 before:h-full before:w-full before:origin-top-left before:scale-x-0 before:bg-[linear-gradient(to_right,var(--shimmer-color)_0%,transparent_100%)] before:transition-transform before:duration-500 before:ease-in-out group-hover:before:scale-x-100",
				className,
			)}
			{...props}
		>
			{children}
		</button>
	);
};
