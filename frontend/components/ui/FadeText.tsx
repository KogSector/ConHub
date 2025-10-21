"use client";

import { cn } from "@/lib/utils";
import { motion, Variants, HTMLMotionProps } from "framer-motion";
import { FC } from "react";

interface FadeTextProps extends HTMLMotionProps<"div"> {
	direction?: "up" | "down" | "left" | "right";
	framerProps?: Variants;
}

export const FadeText: FC<FadeTextProps> = ({
	direction = "up",
	className,
	framerProps = {
		hidden: { opacity: 0 },
		show: { opacity: 1, transition: { type: "spring" } },
	},
	...props
}) => {
	const directionOffset = {
		up: "translateY(-100%)",
		down: "translateY(100%)",
		left: "translateX(-100%)",
		right: "translateX(100%)",
	};

	return (
		<motion.div
			className={cn(
				"relative overflow-hidden",
				className,
			)}
			{...props}
		>
			<motion.div
				variants={framerProps}
				initial="hidden"
				animate="show"
				style={{ transform: directionOffset[direction] }}
			>
				{props.children}
			</motion.div>
		</motion.div>
	);
};
