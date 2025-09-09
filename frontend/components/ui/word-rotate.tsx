"use client";

import { cn } from "@/lib/utils";
import { AnimatePresence, motion } from "framer-motion";
import { FC, HTMLAttributes } from "react";

interface WordRotateProps extends HTMLAttributes<HTMLDivElement> {
	words: string[];
	duration?: number;
	framerProps?: {
		[key: string]: any;
	};
}

export const WordRotate: FC<WordRotateProps> = ({
	words,
	duration = 2500,
	framerProps = {
		initial: { y: "-100%" },
		animate: { y: "0%" },
		exit: { y: "100%" },
		transition: { duration: 0.5, ease: "easeOut" },
	},
	className,
	...props
}) => {
	return (
		<div
			className={cn(
				"relative h-10 w-full overflow-hidden",
				className,
			)}
			{...props}
		>
			<AnimatePresence>
				{words.map((word, i) => (
					<motion.div
						key={word}
						className="absolute inset-0"
						{...framerProps}
					>
						{word}
					</motion.div>
				))}
			</AnimatePresence>
		</div>
	);
};
