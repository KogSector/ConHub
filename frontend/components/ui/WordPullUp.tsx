"use client";

import { cn } from "@/lib/utils";
import { motion, Variants, HTMLMotionProps } from "framer-motion";
import { FC } from "react";

interface WordPullUpProps extends HTMLMotionProps<"h1"> {
	words: string;
	delay?: number;
	wrapperFramerProps?: Variants;
	variants?: Variants;
}

export const WordPullUp: FC<WordPullUpProps> = ({
	words,
	wrapperFramerProps = {
		hidden: { opacity: 0 },
		show: {
			opacity: 1,
			transition: {
				staggerChildren: 0.2,
			},
		},
	},
	variants = {
		hidden: { y: 20, opacity: 0 },
		show: { y: 0, opacity: 1 },
	},
	className,
	...props
}) => {
	return (
		<motion.h1
			variants={wrapperFramerProps}
			initial="hidden"
			animate="show"
			className={cn(
				"text-center font-display text-4xl font-bold tracking-[-0.02em] text-black drop-shadow-sm dark:text-white md:text-4xl md:leading-[5rem]",
				className,
			)}
			{...props}
		>
			{words.split(" ").map((word, i) => (
				<motion.span
					key={i}
					variants={variants}
					style={{ display: "inline-block", paddingRight: "15px" }}
				>
					{word === "" ? <span>&nbsp;</span> : word}
				</motion.span>
			))}
		</motion.h1>
	);
};
