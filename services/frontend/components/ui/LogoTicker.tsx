"use client";

import { cn } from "@/lib/utils";
import { motion } from "framer-motion";
import { FC, HTMLAttributes, ReactNode } from "react";

interface LogoTickerProps extends HTMLAttributes<HTMLDivElement> {
	children: ReactNode;
	duration?: number;
}

export const LogoTicker: FC<LogoTickerProps> = ({
	children,
	duration = 20,
	className,
	...props
}) => {
	return (
		<div
			className={cn(
				"relative flex w-full overflow-hidden",
				className,
			)}
			{...props}
		>
			<motion.div
				className="flex w-max animate-logo-ticker"
				style={{ animationDuration: `${duration}s` }}
			>
				{children}
				{children}
			</motion.div>
		</div>
	);
};
