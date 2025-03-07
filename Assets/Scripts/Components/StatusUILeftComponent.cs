#if UNITY_6000
namespace Assets.Scripts.Components.Unity
{
    using System.Collections.Generic;
    using System.Linq;
    using Assets.Scripts.Components.Core;
    using Assets.Scripts.WorldObjects.Core;
    using Assets.Scripts.WorldObjects.Unity;
    using TMPro;
    using UnityEngine;
    using YamlDotNet.Serialization;
    using YamlDotNet.Serialization.NamingConventions;

    public class StatusUILeftComponent : MonoBehaviour
    {
        // FIELDS //

        private TextMeshProUGUI textMeshPro;
        private ISerializer serializer = new SerializerBuilder()
            .WithNamingConvention(PascalCaseNamingConvention.Instance)
            .ConfigureDefaultValuesHandling(DefaultValuesHandling.OmitNull)
            .Build();

        // FUNCTIONS //

        public void Instantiate()
        {
            this.textMeshPro = this.transform.GetComponent<TextMeshProUGUI>();
        }

        public void Display(IEnumerable<WorldObjectCore> worldObjects)
        {
            // Nothing is here
            if (worldObjects == null)
            {
                this.textMeshPro.SetText("");
            }

            // Get the status data from each object
            IEnumerable<StatusDataComponentCore> statusDataList = worldObjects
                .Where(worldObject => worldObject != null)
                .Where(worldObject => (worldObject.backref as WorldObject) != null)
                .Select(worldObject => worldObject.backref as WorldObject)
                .Select(worldObject => worldObject.StatusData);

            if (statusDataList.Count() == 0)
            {
                // If there are no statuses, clear the status
                this.textMeshPro.SetText("");
            }
            else
            {
                // Serialize the status list to YAML, then update the status
                string statusYaml = this.serializer.Serialize(statusDataList);
                this.textMeshPro.SetText(statusYaml);
            }
        }
    }
}
#endif
