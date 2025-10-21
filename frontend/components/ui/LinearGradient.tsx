"use client";

import { cn } from "@/lib/utils";
import { FC, HTMLAttributes } from "react";

interface LinearGradientProps extends HTMLAttributes<HTMLDivElement> {
	from?: string;
	to?: string;
	direction?: "top" | "bottom" | "left" | "right";
}

export const LinearGradient: FC<LinearGradientProps> = ({
	from = "from-transparent",
	to = "to-black",
	direction = "bottom",
	className,
	...props
}) => {
	const directionClasses = {
		top: "bg-gradient-to-t",
		bottom: "bg-gradient-to-b",
		left: "bg-gradient-to-l",
		right: "bg-gradient-to-r",
	};

	return (
		<div
			className={cn(
				"pointer-events-none absolute inset-0",
				directionClasses[direction],
				from,
				to,
				className,
			)}
			{...props}
		/>
	);
};
