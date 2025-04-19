### Чтобы построить image просто запустить
cd ../drl_omni_peg-main
.docker/build.bash ${TAG:-latest} ${BUILD_ARGS}

После успешного билда image можно запускать контейнер
.docker/run.bash ${TAG:-latest} ${CMD}

Лучше всего на всякий случай смонтировать том с логами куда-нибудь на сервер отдельно.
LOG_VOLUME_HOST="/path/to/custom/logs" .docker/run.bash ${TAG:-latest} ${CMD}

Запуск обучения моделей на 100 million timesteps for training: (not sure if --mode train does anything, but anyway))

cargo run --release --bin ppo -- --mode train
cargo run --release --bin ppo_recurrent -- --mode train
cargo run --release --bin sac -- --mode train
cargo run --release --bin tqc -- --mode train
cargo run --release --bin trpo -- --mode train


### Краткое саммари по тому, что было сделано с кодом от AndrejOrsula

# Требуемая версия Isaac Sim (2023.1.1) вынесена в отдельный image и сохранена как .tar архив в папке DRL_OmniPeg
cd /home/livonian-horde/DRL_OmniPeg
# Загрузка image для Isaac Sim (2023.1.1)
docker load -i isaac_sim_image.tar

# Загрузка image для генерации мешей через blender от автора репозитория
docker load -i blr_procgen.tar

# Тестовая и обучающая выборки, сгенерированные через blr_procgen сохранены по пути /home/livonian-horde/DRL_OmniPeg/Peg-in-hole_Gen_Meshes

### Отредактирован исходный Dockerfile
# 1 Удалена ссылка на Isaac Sim image, чтобы использовалась локально загруженная версия (пришлось вернуть))))

# 2 Вынесены в отдельный RUN установка нужных для OpenUSD библиотек и добавлены три дополнительно
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -yq --no-install-recommends \
    python3 \
    python3-pip \
    cmake \
    curl \
    wget \
    libarchive-dev \
    libgl-dev \
    libglfw3-dev \
    libglib2.0-dev \
    libglu-dev \
    libglu1-mesa-dev \
    libilmbase-dev \
    libssl-dev \
    libtbb2-dev \
    libx11-dev \
    libxt-dev \
    pkg-config \
    python3-dev \
    xz-utils \
    libboost-all-dev \
    tar gzip build-essential \
    build-essential \
    ca-certificates \
    python3-pip \
    clang && \
    python3 -m pip install --no-cache-dir PyOpenGL==3.1.7 pyside6==6.6.0 && \
    rm -rf /var/lib/apt/lists/*
    
# Средства работы с архивами xz-utils и tar gzip build-essential
# Загрузчик wget ( curl вызывает проблемы )
# Попытка использовать альтернативный boost libboost-all-dev (можно удалить наверное, без нее не тестировалось)

# 3 В отдельный RUN вынесена установка нужной версии setuptools==65.5.0
### Install setuptools before OpenUSD build
RUN python3 -m pip install --no-cache-dir setuptools==65.5.0

# 4 В отдельный RUN вынесена установка нужной версии boost_1_76_0.tar.gz (скачивается через wget)
### Install Boost
RUN wget https://sourceforge.net/projects/boost/files/boost/1.76.0/boost_1_76_0.tar.gz -O /tmp/boost_1_76_0.tar.gz && \
    tar -xzf /tmp/boost_1_76_0.tar.gz -C /tmp && \
    cd /tmp/boost_1_76_0 && \
    ./bootstrap.sh && \
    ./b2 install && \
    rm -rf /tmp/boost_1_76_0 /tmp/boost_1_76_0.tar.gz
    
# 5 Авторский патч не применялся, поэтому его копирование в контейнер и все связанные с ним строчки далее закомменчены

# 6 Скачана нужная версия OpenUSD==22.11 локально в /.docker/internal/pxr_sys/OpenUSD-22.11.tar.gz
COPY ./.docker/internal/pxr_sys/OpenUSD-22.11.tar.gz /tmp/OpenUSD-22.11.tar.gz

# 7 В локальный архив с OpenUSD-22.11.tar.gz внесены правки, описанные в авторском патче 
# (см. .docker/internal/pxr_sys/patches/src/build_scripts/build_usd.py.patch)

# 8 Удален аргумент версии OpenUSD, вместо копирования патча в контейнер, туда копируется весь архив с OpenUSD
    
# 9 В OPENUSD_DL_PATH и OPENUSD_SRC_DIR прописана конкретная версия OpenUSD, которая скачано локально
    OPENUSD_DL_PATH="/tmp/OpenUSD-22.11.tar.gz" && \
    OPENUSD_SRC_DIR="/tmp/OpenUSD-22.11" && \
    
# 10 Добавлена проверка, что архив распаковался корректно
# Check if the archive was extracted correctly
    if [ ! -f "${OPENUSD_SRC_DIR}/build_scripts/build_usd.py" ]; then \
        echo "Error: Archive extraction failed or build_usd.py is missing!" && \
        exit 1; \
# 11 При выполнении build_usd.py добавлена опция --jobs 2 чтобы ограничить использование параллельных вычислений (УБРАТЬ ПЕРЕД БИЛДОМ НА СЕРВЕРЕ)

# 12 Отредактирован build.bash добавлены ограничения на ресурсы при билде и проверки использования видеокарты (УБРАТЬ ПЕРЕД БИЛДОМ НА СЕРВЕРЕ)
    --build-arg CUDA_VISIBLE_DEVICES=0 # Restrict to GPU 0
    --build-arg GPU_MEMORY_LIMIT=10240 # Limit GPU memory to 10GB
    --build-arg NVIDIA_VISIBLE_DEVICES=0  # Ensure NVIDIA runtime uses GPU 0
    
# 13 Установка RL зависимостей разделена на 3 отдельных RUN
RUN $ISAAC_SIM_PYTHON_EXE -m pip install --no-cache-dir setuptools==65.5.0 && \
    python3 -m pip install --no-cache-dir setuptools==65.5.0
RUN $ISAAC_SIM_PYTHON_EXE -m pip install --no-cache-dir gymnasium==0.29.1 && \
    python3 -m pip install --no-cache-dir gymnasium==0.29.1
RUN $ISAAC_SIM_PYTHON_EXE -m pip install --no-cache-dir stable-baselines3[extra]==2.2.1 sb3-contrib==2.2.1 && \
    python3 -m pip install --no-cache-dir stable-baselines3[extra]==2.2.1 sb3-contrib==2.2.1
RUN $ISAAC_SIM_PYTHON_EXE -m pip install --no-cache-dir --upgrade "jax[cuda11_pip]" --extra-index-url https://storage.googleapis.com/jax-releases/jax_cuda_releases.html && \
    python3 -m pip install --no-cache-dir --upgrade "jax[cuda11_pip]" --extra-index-url https://storage.googleapis.com/jax-releases/jax_cuda_releases.html


    
