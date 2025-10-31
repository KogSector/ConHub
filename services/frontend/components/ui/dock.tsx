"use client";

import { cn } from "@/lib/utils";
import { motion, useMotionValue, HTMLMotionProps } from "framer-motion";
import { FC, ReactNode, MouseEvent } from "react";

interface DockProps extends HTMLMotionProps<"div"> {
	children: ReactNode;
}

export const Dock: FC<DockProps> = ({ children, className, ...props }) => {
	const mouseX = useMotionValue(Infinity);

	const onMouseMove = (e: MouseEvent<HTMLDivElement>) => {
		mouseX.set(e.pageX);
	};

	const onMouseLeave = () => {
		mouseX.set(Infinity);
	};

	return (
		<motion.div
			onMouseMove={onMouseMove}
			onMouseLeave={onMouseLeave}
			className={cn(
				"flex h-16 items-end gap-4 rounded-2xl bg-black/10 px-4 pb-3",
				"dark:bg-white/10",
				className,
			)}
			{...props}
		>
			{children}
		</motion.div>
	);
};
