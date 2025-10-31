"use client";

import { cn } from "@/lib/utils";
import { motion, MotionProps } from "framer-motion";
import { FC, ReactNode } from "react";

interface MagicButtonProps extends MotionProps {
	children: ReactNode;
	className?: string;
}

export const MagicButton: FC<MagicButtonProps> = ({
	children,
	className,
	...props
}) => {
	return (
		<motion.button
			className={cn(
				"relative inline-flex h-12 w-full min-w-[200px] items-center justify-center overflow-hidden rounded-xl bg-white p-px text-center font-medium text-black",
				"dark:bg-black dark:text-white",
				"focus:outline-none focus:ring-2 focus:ring-slate-400 focus:ring-offset-2 focus:ring-offset-slate-50",
				className,
			)}
			{...props}
		>
			<span className="absolute inset-[-1000%] animate-[spin_2s_linear_infinite] bg-[conic-gradient(from_90deg_at_50%_50%,#E2CBFF_0%,#393BB2_50%,#E2CBFF_100%)]" />
			<span className="inline-flex h-full w-full cursor-pointer items-center justify-center rounded-xl bg-white px-3 py-1 text-sm font-medium text-black backdrop-blur-3xl dark:bg-black dark:text-white">
				{children}
			</span>
		</motion.button>
	);
};
