#!/bin/bash
#
# Slurm 提交脚本：单 GPU 训练 DenseNet 果蔬分类模型
#
# 使用方法：
#   1. 修改下面 SBATCH 资源参数以匹配集群配额
#   2. 确认 TRAIN_DATA_DIR / VAL_DATA_DIR 指向真实数据集
#   3. 在仓库根目录执行： sbatch ai/slurm/train_dense_net.slurm
#
# Slurm 资源配置 -------------------------------------------------------------
#SBATCH --job-name=fruit-train
#SBATCH --output=%x.%j.out
#SBATCH --error=%x.%j.err
#SBATCH --partition=partGPU
#SBATCH --qos=low
#SBATCH --nodes=1
#SBATCH --ntasks-per-node=1
#SBATCH --cpus-per-task=8
#SBATCH --mem=64G
#SBATCH --gres=gpu:1
#SBATCH --time=24:00:00

set -euo pipefail

echo "[$(date '+%F %T')] Job ${SLURM_JOB_ID:-manual} started on $HOSTNAME"

# -----------------------------------------------------------------------------
# 1. 环境准备：按需加载模块 / 创建或复用虚拟环境
# -----------------------------------------------------------------------------
module purge
# 如集群提供特定 CUDA、Python 模块，请在此加载
module load cuda/12.1 || true
module load anaconda3/2024.06 || module load python/3.10 || true

PROJECT_ROOT="${SLURM_SUBMIT_DIR:-$(pwd)}"
ENV_DIR="${PROJECT_ROOT}/ai/.venv"

if [[ ! -d "${ENV_DIR}" ]]; then
  echo "[$(date '+%F %T')] Creating virtualenv at ${ENV_DIR}"
  python3 -m venv "${ENV_DIR}"
fi

source "${ENV_DIR}/bin/activate"
python --version
pip install --upgrade pip
pip install -r "${PROJECT_ROOT}/ai/requirements.txt"

# -----------------------------------------------------------------------------
# 2. 运行参数：可通过 sbatch --export 覆盖，或在此直接修改
# -----------------------------------------------------------------------------
export TRAIN_DATA_DIR="${TRAIN_DATA_DIR:-${PROJECT_ROOT}/ai/Fruit-Images-Dataset-master/Training}"
export VAL_DATA_DIR="${VAL_DATA_DIR:-${PROJECT_ROOT}/ai/Fruit-Images-Dataset-master/test-multiple_fruits}"
export OUTPUT_DIR="${OUTPUT_DIR:-${PROJECT_ROOT}/ai/model/slurm-${SLURM_JOB_ID:-manual}}"
export BATCH_SIZE="${BATCH_SIZE:-128}"
export LEARNING_RATE="${LEARNING_RATE:-0.001}"
export EPOCHS="${EPOCHS:-20}"

mkdir -p "${OUTPUT_DIR}"
echo "[$(date '+%F %T')] Training data: ${TRAIN_DATA_DIR}"
echo "[$(date '+%F %T')] Validation data: ${VAL_DATA_DIR}"
echo "[$(date '+%F %T')] Artifacts dir: ${OUTPUT_DIR}"
echo "[$(date '+%F %T')] Batch size: ${BATCH_SIZE}, LR: ${LEARNING_RATE}, Epochs: ${EPOCHS}"

# -----------------------------------------------------------------------------
# 3. 启动训练
# -----------------------------------------------------------------------------
cd "${PROJECT_ROOT}/ai"

NCCL_DEBUG=${NCCL_DEBUG:-WARN}
PYTHONFAULTHANDLER=1
export NCCL_DEBUG PYTHONFAULTHANDLER

# 如果需要混合精度，可追加 --use-amp 等自定义参数到 train.py
srun python train.py

echo "[$(date '+%F %T')] Job ${SLURM_JOB_ID:-manual} finished"
