"use client";

import { motion, HTMLMotionProps } from "framer-motion";
import { Check, ChevronRight } from "lucide-react";
import { FC, ReactNode } from "react";

interface AnimatedSubscribeButtonProps extends HTMLMotionProps<"button"> {
	buttonTextColor?: string;
	initialText: ReactNode;
	changeText: ReactNode;
	variant?: "default" | "destructive";
}

export const AnimatedSubscribeButton: FC<AnimatedSubscribeButtonProps> = ({
	buttonTextColor,
	initialText,
	changeText,
	variant = "default",
	...props
}) => {
	return (
		<motion.button
			className="relative flex w-[200px] items-center justify-center rounded-md border-2 border-black bg-white p-4 font-semibold"
			style={{
				color: buttonTextColor,
			}}
			{...props}
		>
			<motion.span
				className="absolute inset-0 flex items-center justify-center"
				initial={{ y: 0 }}
				animate={{ y: -50 }}
				transition={{ duration: 0.5 }}
			>
				{initialText}
			</motion.span>
			<motion.span
				className="absolute inset-0 flex items-center justify-center"
				initial={{ y: 50 }}
				animate={{ y: 0 }}
				transition={{ duration: 0.5 }}
			>
				{changeText}
			</motion.span>
		</motion.button>
	);
};
