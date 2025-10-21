"use client";

import { cn } from "@/lib/utils";
import { HTMLAttributes, ReactNode } from "react";

interface RetroButtonProps extends HTMLAttributes<HTMLButtonElement> {
	children: ReactNode;
}

export const RetroButton = ({
	children,
	className,
	...props
}: RetroButtonProps) => {
	return (
		<button
			className={cn(
				"relative inline-flex h-12 items-center justify-center rounded-md border-2 border-black bg-white px-6 font-medium text-black transition-transform active:translate-y-1",
				"dark:border-white dark:bg-black dark:text-white",
				className,
			)}
			{...props}
		>
			<div className="absolute -bottom-2 -right-2 -z-10 h-full w-full rounded-md bg-black dark:bg-white" />
			{children}
		</button>
	);
};
