set -e

cd ../

echo "=== Generate Unity GRPC "
source scripts/generators/unity.sh

generate_unity_grpc_files modules/matches/Realtime/proto/matches.realtime.admin.proto modules/matches/Realtime/client/Unity/games.cheetah.matches.realtime.grpc.admin/Editor/
generate_unity_grpc_files modules/matches/Realtime/proto/matches.realtime.shared.proto modules/matches/Realtime/client/Unity/games.cheetah.matches.realtime.grpc.shared/Runtime/
generate_unity_grpc_files modules/matches/Realtime/proto/matches.realtime.internal.proto modules/matches/Realtime/client/Unity/games.cheetah.matches.realtime.grpc.internal/Runtime/

# мета файлы создаем только для linux - так как
# именно он используется для вывода релиза
if [[ ($OSTYPE == 'linux'*) ]]; then
  generate_meta_files
fi
