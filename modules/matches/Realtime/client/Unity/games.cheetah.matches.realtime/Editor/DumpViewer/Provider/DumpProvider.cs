using System.Threading.Tasks;
using Cheetah.Matches.Realtime.GRPC.Admin;

namespace Cheetah.Matches.Realtime.Editor.DumpViewer.Provider
{
    public interface DumpProvider
    {
        Task<DumpResponse> Dump(ulong room);

        Task Destroy();
    }
}