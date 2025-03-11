namespace Assets.Scripts.UI
{
    using System.Collections.Generic;
    using Assets.Scripts.Core;
    using TMPro;
    using UnityEngine;
    using YamlDotNet.Serialization;
    using YamlDotNet.Serialization.NamingConventions;

    public class StatusUIRight : MonoBehaviour
    {
        // FIELDS //

        private TextMeshProUGUI textMeshPro;
        private ISerializer serializer = new SerializerBuilder()
            .WithNamingConvention(PascalCaseNamingConvention.Instance)
            .ConfigureDefaultValuesHandling(DefaultValuesHandling.OmitNull)
            .Build();

        // FUNCTIONS //

        private class StatusData
        {
            public float Energy { get; set; } = 0;
            public Dictionary<string, uint> Objects { get; set; } = new();
            public Dictionary<string, uint> Resources { get; set; } = new();
        }

        public void Instantiate()
        {
            this.textMeshPro = this.transform.GetComponent<TextMeshProUGUI>();
        }

        public void Display(IEnumerable<WorldObjectCore> worldObjects)
        {
            StatusData data = new();

            if (worldObjects != null)
            {
                foreach (WorldObjectCore worldObject in worldObjects)
                {
                    // Total battery energy, via Energy attribute
                    if (worldObject.battery != null)
                    {
                        data.Energy += worldObject.battery.Energy;
                    }
                    // Count objects
                    string humanizedWorldObjectType = Util.HumanizedString(
                        worldObject.worldObjectType
                    );
                    if (!data.Objects.ContainsKey(humanizedWorldObjectType))
                    {
                        data.Objects.Add(humanizedWorldObjectType, 0);
                    }
                    data.Objects[humanizedWorldObjectType] += 1;
                    // Count resources
                    if (worldObject.resources != null)
                    {
                        foreach (
                            KeyValuePair<string, uint> resource in worldObject.resources.resources
                        )
                        {
                            string humanizedKey = Util.HumanizedString(resource.Key);
                            if (!data.Resources.ContainsKey(humanizedKey))
                            {
                                data.Resources.Add(humanizedKey, 0);
                            }
                            data.Resources[humanizedKey] += resource.Value;
                        }
                    }
                }
            }

            // Serialize the status list to YAML, then update the status
            string statusYaml = this.serializer.Serialize(data);
            this.textMeshPro.SetText(statusYaml);
        }
    }
}
