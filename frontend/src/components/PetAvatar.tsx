import { motion } from 'framer-motion';

type PetAvatarProps = {
  speaking: boolean;
  label: string;
};

export function PetAvatar({ speaking, label }: PetAvatarProps) {
  return (
    <motion.div
      className="pet-avatar"
      animate={{ scale: speaking ? [1, 1.04, 1] : 1 }}
      transition={{ duration: 0.8, repeat: speaking ? Infinity : 0 }}
      aria-label={label}
      role="img"
    >
      <span className="pet-face">◕‿◕</span>
      <span className="pet-name">Claw Mini</span>
    </motion.div>
  );
}
