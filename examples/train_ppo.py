"""Train a PPO agent on a ksim scenario with Stable-Baselines3 and benchmark it.

Needs the `train` extra (`uv sync --extra train`). The evaluation uses the benchmark seeds, so
the written report ranks next to the `ksim-bench run` baselines.
"""

import argparse
from collections.abc import Sequence
from pathlib import Path

import ksim.gym
import numpy as np
import numpy.typing as npt
from ksim.evaluate import evaluate, write_reports
from stable_baselines3 import PPO
from stable_baselines3.common.env_util import make_vec_env


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scenario", default="scenarios/synthetic_step.toml")
    parser.add_argument("--timesteps", type=int, default=200_000)
    parser.add_argument("--num-envs", type=int, default=8)
    parser.add_argument("--seeds", type=int, default=10)
    parser.add_argument("--out", default="results")
    parser.add_argument("--model", default="models/ppo_ksim.zip")
    return parser.parse_args(argv)


def train(scenario: Path, timesteps: int, num_envs: int) -> PPO:
    # SB3 has its own VecEnv API and cannot take a gymnasium VectorEnv, so the vector
    # environment of ksim.gym.make_vec is not used here.
    env = make_vec_env(lambda: ksim.gym.make(scenario), n_envs=num_envs)
    model = PPO("MlpPolicy", env)
    model.learn(total_timesteps=timesteps)
    return model


def main(argv: Sequence[str] | None = None) -> None:
    args = parse_args(argv)
    scenario = Path(args.scenario)
    model = train(scenario, args.timesteps, args.num_envs)
    model_path = Path(args.model)
    model_path.parent.mkdir(parents=True, exist_ok=True)
    model.save(model_path)

    def policy(observation: npt.NDArray[np.float32]) -> npt.ArrayLike:
        return model.predict(observation, deterministic=True)[0]

    reports = evaluate(scenario, policy, "ppo", seeds=range(args.seeds))
    report_path = write_reports(reports, Path(args.out))
    print(f"model: {model_path}\nreport: {report_path}")
    print(
        f"rank against the baselines:\n"
        f"  ksim-bench run --scenario {scenario} --policy k8s --seeds {args.seeds} "
        f"--out {args.out}\n"
        f"  python -m ksim.analysis {args.out} --baseline k8s"
    )


if __name__ == "__main__":
    main()
