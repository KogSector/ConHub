"use client";

import { cn } from "@/lib/utils";
import { motion } from "framer-motion";
import { FC, HTMLAttributes } from "react";

interface LetterPullupProps extends HTMLAttributes<HTMLDivElement> {
	words: string;
	delay?: number;
	y?: number;
}

export const LetterPullup: FC<LetterPullupProps> = ({
	words,
	delay = 0.25,
	y = 20,
	className,
	...props
}) => {
	const letters = words.split("");

	const pullupVariant = {
		initial: { y: y, opacity: 0 },
		animate: (i: any) => ({
			y: 0,
			opacity: 1,
			transition: {
				delay: i * 0.05,
			},
		}),
	};

	return (
		<div className={cn("flex justify-center", className)} {...props}>
			{letters.map((letter, i) => (
				<motion.h1
					key={i}
					variants={pullupVariant}
					initial="initial"
					animate="animate"
					custom={i}
					className="text-center font-display text-4xl font-bold tracking-[-0.02em] text-black drop-shadow-sm dark:text-white md:text-4xl md:leading-[5rem]"
				>
					{letter === " " ? <span>&nbsp;</span> : letter}
				</motion.h1>
			))}
		</div>
	);
};
