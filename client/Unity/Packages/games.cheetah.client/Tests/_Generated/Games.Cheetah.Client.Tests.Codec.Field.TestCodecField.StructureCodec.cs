using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Codec.Formatter;
using Games.Cheetah.Client.Types;
using UnityEngine;
using Games.Cheetah.Client.Tests.Codec.Field;

// ReSharper disable once CheckNamespace
namespace Games_Cheetah_Client_Tests_Codec_Field
{
		// warning warning warning warning warning
		// Code generated by Cheetah relay codec generator - DO NOT EDIT
		// warning warning warning warning warning
		public class TestCodecFieldStructureCodec:Codec<Games.Cheetah.Client.Tests.Codec.Field.TestCodecField.Structure>
		{
			public void Decode(ref CheetahBuffer buffer, ref Games.Cheetah.Client.Tests.Codec.Field.TestCodecField.Structure dest)
			{
				codec0.Decode(ref buffer, ref dest.innerValue);
			}
	
			public void  Encode(in Games.Cheetah.Client.Tests.Codec.Field.TestCodecField.Structure source, ref CheetahBuffer buffer)
			{
				codec0.Encode(in source.innerValue, ref buffer);
			}
	
			private readonly Codec<Games.Cheetah.Client.Tests.Codec.Field.TestCodecField.Inner> codec0;
	
			public TestCodecFieldStructureCodec(CodecRegistry codecRegistry)
			{
				codec0 = codecRegistry.GetCodec<Games.Cheetah.Client.Tests.Codec.Field.TestCodecField.Inner>();
			}
	
	
			[RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.SubsystemRegistration)]
			private static void OnRuntimeMethodLoad()
			{
				CodecRegistryBuilder.RegisterDefault(factory=>new TestCodecFieldStructureCodec(factory));
			}
	
		}
	
	
}